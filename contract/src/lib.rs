#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, token, Address, Env, 
    String, Vec, Symbol
};

#[contracttype]
pub enum DataKey {
    Admin,
    TokenContract,
    PlatformFee,
    FeeRecipient,
    TaskCount,
    Task(u32),
    EscrowBalance(u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[contracttype]
pub enum TaskStatus {
    Open,
    InProgress,
    Completed,
    Disputed,
    Verified,
    Cancelled,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct Task {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub reward: i128,
    pub assignee: Option<Address>,
    pub status: TaskStatus,
    pub created_by: Address,
    pub tags: Vec<String>,
}

#[contract]
pub struct TaskManagerPro;

#[contractimpl]
impl TaskManagerPro {
    /// Initialize the contract with an admin, fee in basis points, token contract, and fee recipient address.
    pub fn initialize(
        env: Env, 
        admin: Address, 
        platform_fee_bps: u32, 
        token_contract: Address,
        fee_recipient: Address
    ) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::PlatformFee, &platform_fee_bps);
        env.storage().instance().set(&DataKey::TokenContract, &token_contract);
        env.storage().instance().set(&DataKey::FeeRecipient, &fee_recipient);
        env.storage().instance().set(&DataKey::TaskCount, &0u32);
    }

    /// Create a task and lock the reward in the contract escrow.
    pub fn create_task(
        env: Env, 
        creator: Address, 
        title: String, 
        description: String, 
        reward: i128, 
        tags: Vec<String>
    ) -> u32 {
        creator.require_auth();
        
        if reward <= 0 {
            panic!("Reward must be positive");
        }

        let task_id: u32 = env.storage().instance().get(&DataKey::TaskCount).unwrap_or(0);
        let token_addr: Address = env.storage().instance().get(&DataKey::TokenContract).expect("Not initialized");
        let token_client = token::Client::new(&env, &token_addr);
        
        // Transfer reward from creator to the contract
        token_client.transfer(&creator, &env.current_contract_address(), &reward);

        let task = Task {
            id: task_id,
            title,
            description,
            reward,
            assignee: None,
            status: TaskStatus::Open,
            created_by: creator,
            tags,
        };
        
        env.storage().instance().set(&DataKey::Task(task_id), &task);
        env.storage().instance().set(&DataKey::EscrowBalance(task_id), &reward);
        env.storage().instance().set(&DataKey::TaskCount, &(task_id + 1));
        
        env.events().publish(
            (Symbol::new(&env, "task_created"), task_id),
            task.clone()
        );

        task_id
    }

    /// Assign an open task to a developer. Any user can claim, or admin can assign.
    pub fn assign_task(env: Env, assignee: Address, task_id: u32) {
        assignee.require_auth();
        
        let mut task: Task = env.storage().instance().get(&DataKey::Task(task_id)).expect("Task not found");
        if task.status != TaskStatus::Open {
            panic!("Task is not open");
        }
        
        task.assignee = Some(assignee.clone());
        task.status = TaskStatus::InProgress;
        env.storage().instance().set(&DataKey::Task(task_id), &task);
        
        env.events().publish(
            (Symbol::new(&env, "task_assigned"), task_id),
            assignee
        );
    }

    /// Mark task as completed (waiting for verification) and supply work delivery details.
    pub fn submit_work(env: Env, assignee: Address, task_id: u32, _delivery_url: String) {
        assignee.require_auth();
        
        let mut task: Task = env.storage().instance().get(&DataKey::Task(task_id)).expect("Task not found");
        if task.status != TaskStatus::InProgress {
            panic!("Task is not in progress");
        }
        
        if Some(assignee.clone()) != task.assignee {
            panic!("Only assignee can submit work");
        }
        
        task.status = TaskStatus::Completed;
        env.storage().instance().set(&DataKey::Task(task_id), &task);
        
        env.events().publish(
            (Symbol::new(&env, "work_submitted"), task_id),
            assignee
        );
    }

    /// Release escrow to the assignee, minus the platform fee. Only admin or task creator can verify and release.
    pub fn complete_task(env: Env, caller: Address, task_id: u32) {
        caller.require_auth();
        
        let mut task: Task = env.storage().instance().get(&DataKey::Task(task_id)).expect("Task not found");
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        
        if caller != task.created_by && caller != admin {
            panic!("Only admin or creator can complete the task");
        }

        if task.status != TaskStatus::Completed {
            panic!("Task is not completed/submitted");
        }
        
        let assignee = task.assignee.clone().expect("Task has no assignee");
        let reward = task.reward;
        
        let platform_fee_bps: u32 = env.storage().instance().get(&DataKey::PlatformFee).unwrap_or(0);
        let fee_recipient: Address = env.storage().instance().get(&DataKey::FeeRecipient).unwrap();
        
        let platform_fee = (reward * platform_fee_bps as i128) / 10000;
        let recipient_amount = reward - platform_fee;
        
        let token_addr: Address = env.storage().instance().get(&DataKey::TokenContract).expect("Not initialized");
        let token_client = token::Client::new(&env, &token_addr);
        
        // Disburse funds
        token_client.transfer(&env.current_contract_address(), &assignee, &recipient_amount);
        if platform_fee > 0 {
            token_client.transfer(&env.current_contract_address(), &fee_recipient, &platform_fee);
        }
        
        task.status = TaskStatus::Verified;
        env.storage().instance().set(&DataKey::Task(task_id), &task);
        env.storage().instance().remove(&DataKey::EscrowBalance(task_id));
        
        env.events().publish(
            (Symbol::new(&env, "task_verified"), task_id),
            reward
        );
    }

    /// Cancel task and refund escrow back to creator. Task must be open.
    pub fn cancel_task(env: Env, creator: Address, task_id: u32) {
        creator.require_auth();
        
        let mut task: Task = env.storage().instance().get(&DataKey::Task(task_id)).expect("Task not found");
        if task.created_by != creator {
            panic!("Only creator can cancel task");
        }
        if task.status != TaskStatus::Open {
            panic!("Task is not open and cannot be cancelled");
        }

        let reward = task.reward;
        let token_addr: Address = env.storage().instance().get(&DataKey::TokenContract).expect("Not initialized");
        let token_client = token::Client::new(&env, &token_addr);
        
        token_client.transfer(&env.current_contract_address(), &creator, &reward);

        task.status = TaskStatus::Cancelled;
        env.storage().instance().set(&DataKey::Task(task_id), &task);
        env.storage().instance().remove(&DataKey::EscrowBalance(task_id));
        
        env.events().publish(
            (Symbol::new(&env, "task_cancelled"), task_id),
            reward
        );
    }

    /// Dispute a task if work is not satisfactory or there is a disagreement.
    pub fn dispute_task(env: Env, caller: Address, task_id: u32) {
        caller.require_auth();
        let mut task: Task = env.storage().instance().get(&DataKey::Task(task_id)).expect("Task not found");
        
        if caller != task.created_by && Some(caller.clone()) != task.assignee {
            panic!("Only creator or assignee can dispute");
        }
        
        if task.status != TaskStatus::InProgress && task.status != TaskStatus::Completed {
            panic!("Can only dispute active or completed tasks");
        }

        task.status = TaskStatus::Disputed;
        env.storage().instance().set(&DataKey::Task(task_id), &task);
        
        env.events().publish(
            (Symbol::new(&env, "task_disputed"), task_id),
            caller
        );
    }

    /// Resolve a dispute. Only admin can resolve and allocate payout split.
    pub fn resolve_dispute(
        env: Env, 
        admin: Address, 
        task_id: u32, 
        creator_refund: i128, 
        assignee_payout: i128
    ) {
        admin.require_auth();
        let actual_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != actual_admin {
            panic!("Only admin can resolve disputes");
        }

        let mut task: Task = env.storage().instance().get(&DataKey::Task(task_id)).expect("Task not found");
        if task.status != TaskStatus::Disputed {
            panic!("Task is not in dispute");
        }

        let total_reward = task.reward;
        if creator_refund + assignee_payout != total_reward {
            panic!("Split must sum to total task reward");
        }

        let token_addr: Address = env.storage().instance().get(&DataKey::TokenContract).expect("Not initialized");
        let token_client = token::Client::new(&env, &token_addr);

        if creator_refund > 0 {
            token_client.transfer(&env.current_contract_address(), &task.created_by, &creator_refund);
        }
        if assignee_payout > 0 {
            let assignee = task.assignee.clone().expect("No assignee");
            
            let platform_fee_bps: u32 = env.storage().instance().get(&DataKey::PlatformFee).unwrap_or(0);
            let fee_recipient: Address = env.storage().instance().get(&DataKey::FeeRecipient).unwrap();
            let platform_fee = (assignee_payout * platform_fee_bps as i128) / 10000;
            let final_assignee_payout = assignee_payout - platform_fee;

            if final_assignee_payout > 0 {
                token_client.transfer(&env.current_contract_address(), &assignee, &final_assignee_payout);
            }
            if platform_fee > 0 {
                token_client.transfer(&env.current_contract_address(), &fee_recipient, &platform_fee);
            }
        }

        task.status = TaskStatus::Verified; // Closed
        env.storage().instance().set(&DataKey::Task(task_id), &task);
        env.storage().instance().remove(&DataKey::EscrowBalance(task_id));

        env.events().publish(
            (Symbol::new(&env, "dispute_resolved"), task_id),
            total_reward
        );
    }

    // Getters
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::Admin)
    }

    pub fn get_token_contract(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::TokenContract)
    }

    pub fn get_platform_fee(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::PlatformFee).unwrap_or(0)
    }

    pub fn get_fee_recipient(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::FeeRecipient)
    }

    pub fn get_task(env: Env, task_id: u32) -> Option<Task> {
        env.storage().instance().get(&DataKey::Task(task_id))
    }

    pub fn get_task_count(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::TaskCount).unwrap_or(0)
    }
}

#[cfg(test)]
mod test;