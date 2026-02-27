#![cfg(test)]

use super::*;
use soroban_sdk::{Env, String, Address, vec, symbol_short};

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TaskManagerContract);
    let client = TaskManagerContractClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    client.initialize(&admin, &100);
    
    // Try to initialize again - should panic
    let result = std::panic::catch_unwind(|| {
        client.initialize(&admin, &200);
    });
    assert!(result.is_err());
    
    println!("✅ Issue #1: Initialization test passed!");
}

#[test]
fn test_create_task() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TaskManagerContract);
    let client = TaskManagerContractClient::new(&env, &contract_id);
    
    // Initialize
    let admin = Address::generate(&env);
    client.initialize(&admin, &100);
    
    // Create task
    let creator = Address::generate(&env);
    let title = String::from_str(&env, "Implement Payment System");
    let description = String::from_str(&env, "Add payment processing with fee deduction");
    let tags = vec![&env, 
        String::from_str(&env, "payment"), 
        String::from_str(&env, "feature")
    ];
    
    let task_id = client.create_task(&creator, &title, &description, &10000, &None, &tags);
    
    assert_eq!(task_id, 0);
    println!("✅ Issue #2: Create task test passed!");
}

#[test]
fn test_assign_task() {
    let env = Env::default();
    let contract_id = env.register_contract(None, TaskManagerContract);
    let client = TaskManagerContractClient::new(&env, &contract_id);
    
    // Setup
    let admin = Address::generate(&env);
    client.initialize(&admin, &100);
    
    let creator = Address::generate(&env);
    let assignee = Address::generate(&env);
    
    let title = String::from_str(&env, "Test Task");
    let description = String::from_str(&env, "Test Description");
    let tags = vec![&env, String::from_str(&env, "test")];
    
    let task_id = client.create_task(&creator, &title, &description, &1000, &None, &tags);
    
    // Assign task
    client.assign_task(&admin, &task_id, &assignee);
    
    println!("✅ Issue #3: Assign task test passed!");
}
