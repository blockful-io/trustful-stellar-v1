use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Map, String, Vec, token};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
enum DataKey {
    ScorerCreator,
    ScorerBadges(String, Address),
    UserScores,
    Managers,
    Initialized,
}

#[contract]
pub struct ScorerContract;

#[contracttype]
#[derive(Debug)]
enum Error {
    ContractAlreadyInitialized,
    Unauthorized,
    ManagerAlreadyExists,
    ManagerNotFound,
    BadgeAlreadyExists,
    BadgeNotFound,
    EmptyBadgeName,
    InvalidIssuer,
}

#[contractimpl]
impl ScorerContract {
    /// Contract constructor
    pub fn initialize(env: Env, scorer_creator: Address) {
        
        // Ensure that the contract is not initialized
        if Self::is_initialized(&env) {
            panic!("{:?}", Error::ContractAlreadyInitialized);
        }

        // Ensure that the scorer creator is the sender
        scorer_creator.require_auth();

        // Store initial state
        env.storage().persistent().set(&DataKey::ScorerCreator, &scorer_creator);
        env.storage().persistent().set(&DataKey::UserScores, &Map::<Address, u32>::new(&env));
        env.storage().persistent().set(&DataKey::Managers, &Vec::<Address>::new(&env));
        env.storage().persistent().set(&DataKey::Initialized, &true);
    }

    
    /// Returns the current version of the contract
    /// 
    /// # Returns
    /// * `u32` - The version number (currently 1)
    pub fn contract_version() -> u32 {
        1
    }

    /// Upgrades the contract's WASM code to a new version
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `new_wasm_hash` - The hash of the new WASM code to upgrade to (32 bytes)
    /// 
    /// # Authorization
    /// * Only the contract admin (scorer_creator) can perform the upgrade
    /// 
    /// # Panics
    /// * If the caller is not the admin
    /// * If the storage operation fails
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = env.storage().persistent().get(&DataKey::ScorerCreator).unwrap();
        admin.require_auth();
        
        env.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Checks if a contract has been initialized
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// 
    /// # Returns
    /// * `bool` - True if the contract is initialized, false otherwise
    fn is_initialized(env: &Env) -> bool {
        env.storage().persistent().get(&DataKey::Initialized).unwrap_or(false)
    }

    /// Retrieves the list of managers and checks if a specific manager exists
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `manager` - The address to check for existence in the managers list
    /// 
    /// # Returns
    /// * `(bool, Vec<Address>)` - A tuple containing:
    ///   - bool: Whether the manager exists in the list
    ///   - Vec<Address>: The complete list of managers
    fn manager_exists(env: &Env, manager: &Address) -> (bool, Vec<Address>) {
        let managers = env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap();
        let exists = managers.iter().any(|m| m == *manager);
        (exists, managers)
    }

    /// Adds a new manager to the contract
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `sender` - The address of the account attempting to add the manager
    /// * `new_manager` - The address of the new manager to be added
    /// 
    /// # Panics
    /// * If the sender is not the scorer creator
    /// * If the manager already exists
    pub fn add_manager(env: Env, sender: Address, new_manager: Address) {
        sender.require_auth();
        
        if sender != env.storage().persistent().get::<DataKey, Address>(&DataKey::ScorerCreator).unwrap() {
            panic!("{:?}", Error::Unauthorized);
        }

        let (exists, mut managers) = Self::manager_exists(&env, &new_manager);
        if exists {
            panic!("{:?}", Error::ManagerAlreadyExists);
        }
        
        managers.push_back(new_manager);
        env.storage().persistent().set(&DataKey::Managers, &managers);
    }

    /// Removes a manager from the contract
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `sender` - The address of the account attempting to remove the manager
    /// * `manager_to_remove` - The address of the manager to be removed
    /// 
    /// # Panics
    /// * If the sender is not the scorer creator
    /// * If the manager does not exist
    pub fn remove_manager(env: Env, sender: Address, manager_to_remove: Address) {
        sender.require_auth();
        
        if sender != env.storage().persistent().get::<DataKey, Address>(&DataKey::ScorerCreator).unwrap() {
            panic!("{:?}", Error::Unauthorized);
        }
        
        let (exists, mut managers) = Self::manager_exists(&env, &manager_to_remove);
        if !exists {
            panic!("{:?}", Error::ManagerNotFound);
        }
        
        if let Some(index) = managers.iter().position(|m| m == manager_to_remove) {
            managers.remove(index as u32);
            env.storage().persistent().set(&DataKey::Managers, &managers);
        }
    }

<<<<<<< HEAD
    /// Adds a new badge to the contract
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `sender` - The address of the account attempting to add the badge
    /// * `issuer` - The address of the issuer of the badge
    /// * `name` - The name of the badge
    /// * `score` - The score of the badge
    pub fn add_badge(env: Env, sender: Address, issuer: Address, name: String, score: u32) {
        sender.require_auth();
        let managers = env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap();
        let is_scorer_creator = sender == env.storage().persistent().get::<DataKey, Address>(&DataKey::ScorerCreator).unwrap();
        let is_manager = managers.iter().any(|m| m == sender);

        // Check if the sender is a manager or the scorer creator
        if !(is_manager || is_scorer_creator) {
            panic!("{:?}", Error::Unauthorized);
        }

        // Validate badge name and issuer
        if name.len() == 0 {
            panic!("{:?}", Error::EmptyBadgeName);
        }
        if issuer.to_string().len() == 0 {
            panic!("{:?}", Error::InvalidIssuer);
        }

        let key = DataKey::ScorerBadges(name, issuer.clone());
        
        if env.storage().persistent().has(&key) {
            panic!("{:?}", Error::BadgeAlreadyExists);
        }

        env.storage().persistent().set(&key, &score);
    }

    /// Removes an existing badge from the contract
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `sender` - The address of the account attempting to remove the badge
    /// * `issuer` - The address of the issuer of the badge
    /// * `name` - The name of the badge to be removed
    pub fn remove_badge(env: Env, sender: Address, issuer: Address, name: String) {
        sender.require_auth();
        let managers = env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap();
        let is_scorer_creator = sender == env.storage().persistent().get::<DataKey, Address>(&DataKey::ScorerCreator).unwrap();
        let is_manager = managers.iter().any(|m| m == sender);

        // Check if the sender is a manager or the scorer creator
        if !(is_manager || is_scorer_creator) {
            panic!("{:?}", Error::Unauthorized);
        }

        // Validate badge name and issuer
        if name.len() == 0 {
            panic!("{:?}", Error::EmptyBadgeName);
        }
        if issuer.to_string().len() == 0 {
            panic!("{:?}", Error::InvalidIssuer);
        }

        let key = DataKey::ScorerBadges(name, issuer.clone());
        
        if !env.storage().persistent().has(&key) {
            panic!("{:?}", Error::BadgeNotFound);
        }

        env.storage().persistent().remove(&key);
    }

    pub fn check_balance(env: Env, user: Address, token_address: Address) -> i128 {
        let client = token::Client::new(&env, &token_address);
        client.balance(&user)
=======
    /// Retrieves the score of a user
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `user` - The address of the user to retrieve the score for
    /// 
    /// # Returns
    /// * `u32` - The score of the user, or 0 if the user has no score
    pub fn get_user_score(env: Env, user: Address) -> u32 {
        let user_scores = env.storage().persistent().get::<DataKey, Map<Address, u32>>(&DataKey::UserScores).unwrap();
        user_scores.get(user).unwrap_or(0)
>>>>>>> origin/main
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use crate::test_utils::{old_contract, new_contract};

    fn setup_contract() -> (Env, Address, ScorerContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();

        // Variables to initialize the contract
        let scorer_creator = Address::generate(&env);

        // Register the contract
        let scorer_contract_id = env.register_contract(None, ScorerContract);
        let scorer_client = ScorerContractClient::new(&env, &scorer_contract_id);

        // Initialize contract
        scorer_client.initialize(&scorer_creator);

        (env, scorer_creator, scorer_client)
    }

    #[test]
    fn test_initialize() {
        setup_contract();
    }

    #[test]
    #[should_panic(expected = "ContractAlreadyInitialized")]
    fn test_double_initialization() {
        let (_env, scorer_creator, client) = setup_contract();
        
        client.initialize(&scorer_creator);
    }

    #[test]
    fn test_add_manager() {
        let (env, scorer_creator, client) = setup_contract();
        let new_manager = Address::generate(&env);
        client.add_manager(&scorer_creator, &new_manager);

        let managers = env.as_contract(&client.address, || {
            env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap()
        });
        
        assert_eq!(managers, Vec::from_slice(&env, &[new_manager]));
    }

    #[test]
    fn test_remove_manager() {
        let (env, scorer_creator, client) = setup_contract();
        let new_manager = Address::generate(&env);
        client.add_manager(&scorer_creator, &new_manager);
        client.remove_manager(&scorer_creator, &new_manager);

        let managers = env.as_contract(&client.address, || {
            env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap()
        });
        
        assert_eq!(managers, Vec::<Address>::new(&env));
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_add_manager_unauthorized() {
        let (env, _scorer_creator, client) = setup_contract();
        let unauthorized_user = Address::generate(&env);
        let new_manager = Address::generate(&env);
        
        client.add_manager(&unauthorized_user, &new_manager);
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_remove_manager_unauthorized() {
        let (env, _scorer_creator, client) = setup_contract();
        let unauthorized_user = Address::generate(&env);
        
        client.remove_manager(&unauthorized_user, &unauthorized_user);
    }

    #[test]
    fn test_multiple_managers() {
        let (env, scorer_creator, client) = setup_contract();
        let manager1 = Address::generate(&env);
        let manager2 = Address::generate(&env);
        let manager3 = Address::generate(&env);

        client.add_manager(&scorer_creator, &manager1);
        client.add_manager(&scorer_creator, &manager2);
        client.add_manager(&scorer_creator, &manager3);

        let managers = env.as_contract(&client.address, || {
            env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap()
        });
        
        assert_eq!(managers, Vec::from_slice(&env, &[manager1.clone(), manager2.clone(), manager3.clone()]));

        client.remove_manager(&scorer_creator, &manager2);

        let managers_after_remove = env.as_contract(&client.address, || {
            env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap()
        });
        
        assert_eq!(managers_after_remove, Vec::from_slice(&env, &[manager1, manager3]));
    }

    #[test]
    fn test_upgrade() {
        let (env, _scorer_creator, client) = setup_contract();
        assert_eq!(1, client.contract_version());
        let new_wasm_hash = env.deployer().upload_contract_wasm(old_contract::WASM);
        client.upgrade(&new_wasm_hash);

        assert_eq!(0, client.contract_version());
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_upgrade_unauthorized() {
        let (env, _scorer_creator, client) = setup_contract();
        let new_wasm_hash = env.deployer().upload_contract_wasm(new_contract::WASM);
        env.mock_auths(&[]);
        client.upgrade(&new_wasm_hash);
    }

    #[test]
<<<<<<< HEAD
    fn test_remove_badge() {
        let (env, scorer_creator, client) = setup_contract();
        let issuer = Address::generate(&env);
        let badge_name = String::from_str(&env, "TestBadge");
        let score = 100;

        // Add a badge first
        client.add_badge(&scorer_creator, &issuer, &badge_name, &score);

        // Remove the badge
        client.remove_badge(&scorer_creator, &issuer, &badge_name);

        // Check that the badge no longer exists
        let key = DataKey::ScorerBadges(badge_name.clone(), issuer.clone());
        let badge_exists = env.as_contract(&client.address, || {
            env.storage().persistent().has(&key)
        });

        assert!(!badge_exists, "Badge should be removed");
    }

    #[test]
    #[should_panic(expected = "BadgeNotFound")]
    fn test_remove_nonexistent_badge() {
        let (env, scorer_creator, client) = setup_contract();
        let issuer = Address::generate(&env);
        let badge_name = String::from_str(&env, "NonExistentBadge");

        // Attempt to remove a badge that doesn't exist
        client.remove_badge(&scorer_creator, &issuer, &badge_name);
=======
    fn test_get_user_score() {
        let (env, _scorer_creator, client) = setup_contract();
        let user = Address::generate(&env);
        assert_eq!(0, client.get_user_score(&user));
>>>>>>> origin/main
    }
}   