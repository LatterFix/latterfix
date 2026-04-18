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
    TaskCount,
    Task(u32),
    EscrowBalance(u32),
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum TaskStatus {
    Open,
    InProgress,
    Completed,
    Disputed,
}

#[derive(Clone)]
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
    /// Initialize the contract with an admin, fee in basis points, and a token contract (e.g., USDC or XLM).
    pub fn initialize(env: Env, admin: Address, platform_fee_bps: u32, token_contract: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::PlatformFee, &platform_fee_bps);
        env.storage().instance().set(&DataKey::TokenContract, &token_contract);
        env.storage().instance().set(&DataKey::TaskCount, &0u32);
    }

    /// Create a task, requiring the creator to transfer `reward` tokens in to the contract as escrow.
    pub fn create_task(env: Env, creator: Address, title: String, description: String, reward: i128, tags: Vec<String>) -> u32 {
        creator.require_auth();
        
        if reward <= 0 {
            panic!("Reward must be positive");
        }

        let task_id: u32 = env.storage().instance().get(&DataKey::TaskCount).unwrap_or(0);
        
        let token_addr: Address = env.storage().instance().get(&DataKey::TokenContract).expect("Not initialized");
        let token_client = token::Client::new(&env, &token_addr);
        
        // Use Stellar's native token transfers to lock the funds in the contract.
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

    /// Assign an active task to a user. Only admin can assign.
    pub fn assign_task(env: Env, admin: Address, task_id: u32, assignee: Address) {
        admin.require_auth();
        let actual_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != actual_admin {
            panic!("Only admin can assign tasks");
        }
        
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

    /// Complete a task and release the securely escrowed Stellar tokens to the assignee's balance.
    pub fn complete_task(env: Env, admin: Address, task_id: u32) {
        admin.require_auth();
        let actual_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if admin != actual_admin {
            panic!("Only admin can complete tasks");
        }
        
        let mut task: Task = env.storage().instance().get(&DataKey::Task(task_id)).expect("Task not found");
        if task.status != TaskStatus::InProgress {
            panic!("Task is not currently strictly in-progress");
        }
        
        let assignee = task.assignee.clone().expect("Task has no assignee");
        let reward = task.reward;
        let platform_fee_bps: u32 = env.storage().instance().get(&DataKey::PlatformFee).unwrap_or(0);
        
        // Fee calculations
        let platform_fee = (reward * platform_fee_bps as i128) / 10000;
        let recipient_amount = reward - platform_fee;
        
        let token_addr: Address = env.storage().instance().get(&DataKey::TokenContract).expect("Not initialized");
        let token_client = token::Client::new(&env, &token_addr);
        
        // Issue payout 
        token_client.transfer(&env.current_contract_address(), &assignee, &recipient_amount);
        if platform_fee > 0 {
            token_client.transfer(&env.current_contract_address(), &admin, &platform_fee);
        }
        
        task.status = TaskStatus::Completed;
        env.storage().instance().set(&DataKey::Task(task_id), &task);
        env.storage().instance().remove(&DataKey::EscrowBalance(task_id));
        
        env.events().publish(
            (Symbol::new(&env, "task_completed"), task_id),
            reward
        );
    }

    /// View a task
    pub fn get_task(env: Env, task_id: u32) -> Option<Task> {
        env.storage().instance().get(&DataKey::Task(task_id))
    }

    /// View all tasks (paginated)
    pub fn get_tasks(env: Env, start_id: u32, limit: u32) -> Vec<Task> {
        let task_count: u32 = env.storage().instance().get(&DataKey::TaskCount).unwrap_or(0);
        let mut tasks = Vec::new(&env);
        let end = (start_id + limit).min(task_count);
        let mut i = start_id;
        while i < end {
            if let Some(task) = env.storage().instance().get::<DataKey, Task>(&DataKey::Task(i)) {
                tasks.push_back(task);
            }
            i += 1;
        }
        tasks
    }

    /// View total task count
    pub fn get_task_count(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::TaskCount).unwrap_or(0)
    }
}