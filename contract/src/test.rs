#![cfg(test)]

use super::*;
use soroban_sdk::{Env, String, Address, vec, testutils::Address as _};

fn setup_test_env(env: &Env) -> (Address, Address, Address, Address, TaskManagerProClient<'static>) {
    let contract_id = env.register_contract(None, TaskManagerPro);
    let client = TaskManagerProClient::new(env, &contract_id);
    
    let admin = Address::generate(env);
    let fee_recipient = Address::generate(env);
    let token_admin = Address::generate(env);
    
    // Register Stellar mock token
    let token_address = env.register_stellar_asset_contract(token_admin.clone());
    
    env.mock_all_auths();

    // Initialize TaskManagerPro contract with 2.5% platform fee (250 BPS)
    client.initialize(&admin, &250, &token_address, &fee_recipient);
    
    (admin, fee_recipient, token_address, contract_id, client)
}

#[test]
fn test_initialization() {
    let env = Env::default();
    let (admin, fee_recipient, token_address, _, client) = setup_test_env(&env);
    
    assert_eq!(client.get_admin().unwrap(), admin);
    assert_eq!(client.get_token_contract().unwrap(), token_address);
    assert_eq!(client.get_platform_fee(), 250);
    assert_eq!(client.get_fee_recipient().unwrap(), fee_recipient);
    assert_eq!(client.get_task_count(), 0);
}

#[test]
fn test_create_and_complete_task_flow() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (admin, fee_recipient, token_address, contract_id, client) = setup_test_env(&env);
    let token_client = token::Client::new(&env, &token_address);
    
    // Fetch token admin to create a StellarAssetClient for minting
    // We register the asset admin client to mint
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    
    let creator = Address::generate(&env);
    let assignee = Address::generate(&env);
    
    // Mint tokens to creator so they can fund the task
    let reward = 10000i128;
    token_admin_client.mint(&creator, &reward);
    assert_eq!(token_client.balance(&creator), reward);
    
    // Create Task
    let title = String::from_str(&env, "Build stellar UI");
    let description = String::from_str(&env, "Integrate freighter wallet interface.");
    let tags = vec![&env, String::from_str(&env, "stellar"), String::from_str(&env, "nextjs")];
    
    let task_id = client.create_task(&creator, &title, &description, &reward, &tags);
    assert_eq!(task_id, 0);
    
    // Creator funds are locked inside the escrow (contract)
    assert_eq!(token_client.balance(&creator), 0);
    assert_eq!(token_client.balance(&contract_id), reward);
    
    // Assign Task
    client.assign_task(&assignee, &task_id);
    let task = client.get_task(&task_id).unwrap();
    assert_eq!(task.status, TaskStatus::InProgress);
    assert_eq!(task.assignee.unwrap(), assignee);
    
    // Submit Work
    client.submit_work(&assignee, &task_id, &String::from_str(&env, "http://github.com/pr/1"));
    let task_after_submission = client.get_task(&task_id).unwrap();
    assert_eq!(task_after_submission.status, TaskStatus::Completed);
    
    // Complete Task (Verifying payout and fee cut)
    client.complete_task(&admin, &task_id);
    
    // 2.5% fee of 10000 = 250
    // Assignee receives 9750
    // Fee recipient receives 250
    assert_eq!(token_client.balance(&assignee), 9750);
    assert_eq!(token_client.balance(&fee_recipient), 250);
    assert_eq!(token_client.balance(&contract_id), 0);
    
    let finished_task = client.get_task(&task_id).unwrap();
    assert_eq!(finished_task.status, TaskStatus::Verified);
}

#[test]
fn test_cancel_task_refund() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (_, _, token_address, contract_id, client) = setup_test_env(&env);
    let token_client = token::Client::new(&env, &token_address);
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    
    let creator = Address::generate(&env);
    let reward = 5000i128;
    token_admin_client.mint(&creator, &reward);
    
    let title = String::from_str(&env, "Simple Design Task");
    let description = String::from_str(&env, "Figma design required.");
    let tags = vec![&env, String::from_str(&env, "design")];
    
    let task_id = client.create_task(&creator, &title, &description, &reward, &tags);
    assert_eq!(token_client.balance(&contract_id), reward);
    
    // Cancel Task and verify refund
    client.cancel_task(&creator, &task_id);
    
    assert_eq!(token_client.balance(&creator), reward);
    assert_eq!(token_client.balance(&contract_id), 0);
    
    let task = client.get_task(&task_id).unwrap();
    assert_eq!(task.status, TaskStatus::Cancelled);
}

#[test]
fn test_dispute_and_resolution() {
    let env = Env::default();
    env.mock_all_auths();
    
    let (admin, fee_recipient, token_address, contract_id, client) = setup_test_env(&env);
    let token_client = token::Client::new(&env, &token_address);
    let token_admin_client = soroban_sdk::token::StellarAssetClient::new(&env, &token_address);
    
    let creator = Address::generate(&env);
    let assignee = Address::generate(&env);
    let reward = 20000i128;
    token_admin_client.mint(&creator, &reward);
    
    let task_id = client.create_task(
        &creator, 
        &String::from_str(&env, "Disputed Bounty"), 
        &String::from_str(&env, "Will raise a dispute."), 
        &reward, 
        &vec![&env]
    );
    
    client.assign_task(&assignee, &task_id);
    
    // Creator disputes active task
    client.dispute_task(&creator, &task_id);
    let task = client.get_task(&task_id).unwrap();
    assert_eq!(task.status, TaskStatus::Disputed);
    
    // Admin resolves dispute: 50/50 split (10000 creator refund, 10000 assignee payout)
    // Assignee receives 10000 minus 2.5% fee (250) = 9750
    // Fee recipient receives 250
    // Creator receives 10000 refund
    client.resolve_dispute(&admin, &task_id, &10000, &10000);
    
    assert_eq!(token_client.balance(&creator), 10000);
    assert_eq!(token_client.balance(&assignee), 9750);
    assert_eq!(token_client.balance(&fee_recipient), 250);
    assert_eq!(token_client.balance(&contract_id), 0);
    
    let task_resolved = client.get_task(&task_id).unwrap();
    assert_eq!(task_resolved.status, TaskStatus::Verified);
}

#[test]
fn test_user_profile_lifecycle() {
    let env = Env::default();
    env.mock_all_auths();
    
    let user = Address::generate(&env);
    let admin = Address::generate(&env);
    
    // Register UserProfileManager contract
    let contract_id = env.register_contract(None, user_profile::UserProfileManager);
    let client = user_profile::UserProfileManagerClient::new(&env, &contract_id);
    
    // Create profile
    let username = String::from_str(&env, "stellar_dev");
    let bio = String::from_str(&env, "Soroban Smart Contract Developer");
    client.create_profile(&user, &username, &bio);
    
    // Retrieve profile and verify details
    let profile = client.get_profile(&user).unwrap();
    assert_eq!(profile.username, username);
    assert_eq!(profile.bio, bio);
    assert_eq!(profile.reputation, 100);
    assert_eq!(profile.completed_tasks, 0);
    
    // Update bio
    let new_bio = String::from_str(&env, "Expert Rust & Soroban Engineer");
    client.update_bio(&user, &new_bio);
    assert_eq!(client.get_profile(&user).unwrap().bio, new_bio);
    
    // Reward contribution (reputation increment)
    client.reward_contribution(&admin, &user, &50);
    let updated = client.get_profile(&user).unwrap();
    assert_eq!(updated.reputation, 150);
    assert_eq!(updated.completed_tasks, 1);
}