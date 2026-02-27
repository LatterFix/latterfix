#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, Env, String, Vec, Map, Symbol};

// ==================== DATA STRUCTURES ====================
#[derive(Clone)]
#[contracttype]
pub enum DataKey {
    Admin,
    Task(u32),
    UserTask(Address, u32),
    TaskCounter,
    UserProfile(Address),
    PlatformFee,
    Paused,
    TaskReward(u32),
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
    pub created_at: u64,
    pub deadline: Option<u64>,
    pub tags: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
#[contracttype]
pub enum TaskStatus {
    Open,
    Assigned,
    InProgress,
    Completed,
    Verified,
    Cancelled,
}

#[derive(Clone)]
#[contracttype]
pub struct UserProfile {
    pub address: Address,
    pub username: Option<String>,
    pub reputation: u32,
    pub completed_tasks: u32,
    pub joined_at: u64,
    pub bio: Option<String>,
}

#[derive(Clone)]
#[contracttype]
pub struct TaskReward {
    pub task_id: u32,
    pub amount: i128,
    pub claimed: bool,
    pub released_at: Option<u64>,
}

// ==================== CONTRACT ====================
#[contract]
pub struct TaskManagerContract;

#[contractimpl]
impl TaskManagerContract {
    // ===== ISSUE #1: Initialize contract with admin and platform fee =====
    pub fn initialize(env: Env, admin: Address, platform_fee_bps: u32) {
        // Check if already initialized
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        
        // Set admin
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        
        // Set platform fee (basis points, e.g., 100 = 1%)
        env.storage().instance().set(&DataKey::PlatformFee, &platform_fee_bps);
        
        // Initialize task counter
        env.storage().instance().set(&DataKey::TaskCounter, &0u32);
        
        // Set initial paused state
        env.storage().instance().set(&DataKey::Paused, &false);
    }
    
    // ===== ISSUE #2: Create a new task with validation =====
    pub fn create_task(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        reward: i128,
        deadline: Option<u64>,
        tags: Vec<String>,
    ) -> u32 {
        // Check if contract is paused
        if Self::is_paused(&env) {
            panic!("Contract is paused");
        }
        
        // Validate inputs
        if reward <= 0 {
            panic!("Reward must be positive");
        }
        
        if title.len() == 0 || title.len() > 100 {
            panic!("Title must be between 1-100 characters");
        }
        
        if description.len() > 5000 {
            panic!("Description too long (max 5000 chars)");
        }
        
        // Authenticate creator
        creator.require_auth();
        
        // Get current task counter and increment
        let mut counter: u32 = env.storage().instance().get(&DataKey::TaskCounter).unwrap_or(0);
        let task_id = counter;
        counter += 1;
        env.storage().instance().set(&DataKey::TaskCounter, &counter);
        
        // Create task
        let task = Task {
            id: task_id,
            title,
            description,
            reward,
            assignee: None,
            status: TaskStatus::Open,
            created_by: creator.clone(),
            created_at: env.ledger().timestamp(),
            deadline,
            tags,
        };
        
        // Store task
        env.storage().instance().set(&DataKey::Task(task_id), &task);
        
        // Store task reward info
        let task_reward = TaskReward {
            task_id,
            amount: reward,
            claimed: false,
            released_at: None,
        };
        env.storage().instance().set(&DataKey::TaskReward(task_id), &task_reward);
        
        task_id
    }
    
    // ===== ISSUE #3: Assign task to a contributor =====
    pub fn assign_task(env: Env, admin: Address, task_id: u32, assignee: Address) {
        // Verify admin
        Self::require_admin(&env, admin);
        
        // Get task
        let mut task: Task = env.storage().instance()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");
        
        // Check task status
        if task.status != TaskStatus::Open {
            panic!("Task is not open for assignment");
        }
        
        // Update task
        task.assignee = Some(assignee.clone());
        task.status = TaskStatus::Assigned;
        
        // Save task
        env.storage().instance().set(&DataKey::Task(task_id), &task);
        
        // Link user to task
        env.storage().instance().set(&DataKey::UserTask(assignee, task_id), &task_id);
    }
    
    // ===== ISSUE #4: Update task status with workflow validation =====
    pub fn update_task_status(env: Env, caller: Address, task_id: u32, new_status: TaskStatus) {
        // Get task
        let mut task: Task = env.storage().instance()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");
        
        // Verify authorization based on action
        match new_status {
            TaskStatus::InProgress => {
                // Only assignee can mark as in progress
                if Some(caller.clone()) != task.assignee {
                    panic!("Only assignee can start task");
                }
                caller.require_auth();
                
                // Check if previous status allows transition
                if task.status != TaskStatus::Assigned {
                    panic!("Task must be assigned first");
                }
            },
            TaskStatus::Completed => {
                // Only assignee can mark as completed
                if Some(caller.clone()) != task.assignee {
                    panic!("Only assignee can complete task");
                }
                caller.require_auth();
                
                // Check if previous status allows transition
                if task.status != TaskStatus::InProgress {
                    panic!("Task must be in progress first");
                }
            },
            TaskStatus::Verified => {
                // Only admin can verify completion
                Self::require_admin(&env, caller);
                
                // Check if previous status allows transition
                if task.status != TaskStatus::Completed {
                    panic!("Task must be completed first");
                }
            },
            TaskStatus::Cancelled => {
                // Admin or creator can cancel
                if caller != task.created_by && !Self::is_admin(&env, caller.clone()) {
                    panic!("Only creator or admin can cancel");
                }
                caller.require_auth();
                
                // Can cancel any open/assigned task
                if task.status == TaskStatus::Completed || task.status == TaskStatus::Verified {
                    panic!("Cannot cancel completed task");
                }
            },
            _ => panic!("Invalid status transition"),
        }
        
        // Update task status
        task.status = new_status;
        env.storage().instance().set(&DataKey::Task(task_id), &task);
    }
    
    // ===== ISSUE #5: Release payment to assignee with platform fee deduction =====
    pub fn release_payment(env: Env, admin: Address, task_id: u32) {
        // Only admin can release payment
        Self::require_admin(&env, admin);
        
        // Get task
        let task: Task = env.storage().instance()
            .get(&DataKey::Task(task_id))
            .expect("Task not found");
        
        // Verify task is verified
        if task.status != TaskStatus::Verified {
            panic!("Task must be verified to release payment");
        }
        
        // Get task reward info
        let mut task_reward: TaskReward = env.storage().instance()
            .get(&DataKey::TaskReward(task_id))
            .expect("Task reward not found");
        
        // Check if already claimed
        if task_reward.claimed {
            panic!("Payment already released");
        }
        
        // Calculate platform fee
        let platform_fee_bps: u32 = env.storage().instance()
            .get(&DataKey::PlatformFee)
            .unwrap_or(0);
        
        let platform_fee = (task.reward * platform_fee_bps as i128) / 10000;
        let assignee_reward = task.reward - platform_fee;
        
        // Transfer platform fee to admin (simplified - actual token transfer would need token contract)
        // In real implementation, you'd call token contract's transfer function
        
        // Transfer reward to assignee (simplified)
        if let Some(assignee) = task.assignee {
            // Simulate payment release
            // In real implementation: token_client.transfer(&env.current_contract_address(), &assignee, &assignee_reward);
            
            // Update task reward
            task_reward.claimed = true;
            task_reward.released_at = Some(env.ledger().timestamp());
            env.storage().instance().set(&DataKey::TaskReward(task_id), &task_reward);
            
            // Update user reputation
            Self::update_user_reputation(&env, assignee, 10);
        } else {
            panic!("No assignee for this task");
        }
    }
    
    // ===== ISSUE #6: Create/update user profile =====
    pub fn upsert_user_profile(
        env: Env,
        user: Address,
        username: Option<String>,
        bio: Option<String>,
    ) {
        user.require_auth();
        
        // Get existing profile or create new
        let mut profile: UserProfile = match env.storage().instance().get(&DataKey::UserProfile(user.clone())) {
            Some(profile) => profile,
            None => UserProfile {
                address: user.clone(),
                username: None,
                reputation: 0,
                completed_tasks: 0,
                joined_at: env.ledger().timestamp(),
                bio: None,
            },
        };
        
        // Update fields
        if let Some(new_username) = username {
            // Validate username
            if new_username.len() < 3 || new_username.len() > 30 {
                panic!("Username must be 3-30 characters");
            }
            profile.username = Some(new_username);
        }
        
        if let Some(new_bio) = bio {
            if new_bio.len() > 500 {
                panic!("Bio too long (max 500 chars)");
            }
            profile.bio = Some(new_bio);
        }
        
        // Save profile
        env.storage().instance().set(&DataKey::UserProfile(user), &profile);
    }
    
    // ===== ISSUE #7: Get task details with error handling =====
    pub fn get_task(env: Env, task_id: u32) -> Option<Task> {
        env.storage().instance().get(&DataKey::Task(task_id))
    }
    
    // ===== ISSUE #8: Get user profile with stats =====
    pub fn get_user_profile(env: Env, user: Address) -> Option<UserProfile> {
        env.storage().instance().get(&DataKey::UserProfile(user))
    }
    
    // ===== ISSUE #9: Emergency pause/unpause functionality =====
    pub fn set_paused(env: Env, admin: Address, paused: bool) {
        Self::require_admin(&env, admin);
        env.storage().instance().set(&DataKey::Paused, &paused);
    }
    
    // ===== ISSUE #10: Get all tasks by status with pagination =====
    pub fn get_tasks_by_status(env: Env, status: TaskStatus, page: u32, page_size: u32) -> Vec<Task> {
        let counter: u32 = env.storage().instance().get(&DataKey::TaskCounter).unwrap_or(0);
        let mut tasks = Vec::new(&env);
        let start = page * page_size;
        let end = (start + page_size).min(counter);
        
        for task_id in start..end {
            if let Some(task) = Self::get_task(env.clone(), task_id) {
                if task.status == status {
                    tasks.push_back(task);
                }
            }
        }
        
        tasks
    }
    
    // ===== HELPER FUNCTIONS =====
    
    // Check if contract is paused
    fn is_paused(env: &Env) -> bool {
        env.storage().instance().get(&DataKey::Paused).unwrap_or(false)
    }
    
    // Check if address is admin
    fn is_admin(env: &Env, address: Address) -> bool {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin == address
    }
    
    // Require admin authorization
    fn require_admin(env: &Env, admin: Address) {
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        if stored_admin != admin {
            panic!("Not authorized");
        }
        admin.require_auth();
    }
    
    // Update user reputation
    fn update_user_reputation(env: &Env, user: Address, points: u32) {
        if let Some(mut profile) = Self::get_user_profile(env.clone(), user.clone()) {
            profile.reputation += points;
            profile.completed_tasks += 1;
            env.storage().instance().set(&DataKey::UserProfile(user), &profile);
        }
    }
}

// ==================== TESTS MODULE ====================
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{vec, Env, String};
    
    #[test]
    fn test_issue_1_initialize() {
        let env = Env::default();
        let contract_id = env.register_contract(None, TaskManagerContract);
        let client = TaskManagerContractClient::new(&env, &contract_id);
        
        let admin = Address::random(&env);
        client.initialize(&admin, &100);
        
        // Verify initialization by trying to create task (should not panic)
        // This would require additional setup
    }
    
    // Additional tests would be added for each issue
}
