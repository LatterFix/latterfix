use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[contracttype]
pub enum UserKey {
    Profile(Address),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserProfile {
    pub address: Address,
    pub username: String,
    pub reputation: u32,
    pub completed_tasks: u32,
    pub joined_at: u64,
    pub bio: String,
}

#[contract]
pub struct UserProfileManager;

#[contractimpl]
impl UserProfileManager {
    /// Create a new developer profile on-chain.
    pub fn create_profile(env: Env, user: Address, username: String, bio: String) {
        user.require_auth();
        let key = UserKey::Profile(user.clone());
        if env.storage().instance().has(&key) {
            panic!("Profile already exists");
        }
        let profile = UserProfile {
            address: user.clone(),
            username,
            reputation: 100, // starting reputation points
            completed_tasks: 0,
            joined_at: env.ledger().timestamp(),
            bio,
        };
        env.storage().instance().set(&key, &profile);
    }

    /// Update bio details of the contributor.
    pub fn update_bio(env: Env, user: Address, new_bio: String) {
        user.require_auth();
        let key = UserKey::Profile(user.clone());
        let mut profile: UserProfile = env.storage().instance().get(&key).expect("Profile not found");
        profile.bio = new_bio;
        env.storage().instance().set(&key, &profile);
    }

    /// Increment reputation points when a verified escrow is completed.
    pub fn reward_contribution(env: Env, contract_admin: Address, user: Address, points: u32) {
        contract_admin.require_auth();
        let key = UserKey::Profile(user.clone());
        let mut profile: UserProfile = env.storage().instance().get(&key).expect("Profile not found");
        profile.reputation += points;
        profile.completed_tasks += 1;
        env.storage().instance().set(&key, &profile);
    }

    /// Retrieve contributor profile details.
    pub fn get_profile(env: Env, user: Address) -> Option<UserProfile> {
        let key = UserKey::Profile(user);
        env.storage().instance().get(&key)
    }
}
