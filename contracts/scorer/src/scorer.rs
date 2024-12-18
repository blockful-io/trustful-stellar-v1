#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Map, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScorerBadge {
    pub name: String,
    pub issuer: Address,
    pub score: u32,
}

#[contracttype]
enum DataKey {
    ScorerCreator,
    ScorerBadges,
    Users,
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
}

#[contractimpl]
impl ScorerContract {
    /// Contract constructor
    pub fn initialize(env: Env, scorer_creator: Address, scorer_badges: Map<u32, ScorerBadge>) {
        
        // Ensure that the contract is not initialized
        if Self::is_initialized(&env) {
            panic!("{:?}", Error::ContractAlreadyInitialized);
        }

        // Ensure that the scorer creator is the sender
        scorer_creator.require_auth();

        // Store initial state
        env.storage().persistent().set(&DataKey::ScorerCreator, &scorer_creator);
        env.storage().persistent().set(&DataKey::ScorerBadges, &scorer_badges);
        env.storage().persistent().set(&DataKey::Users, &Map::<Address, bool>::new(&env));
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

    
    /// Adds a new user to the contract's user registry
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `user` - The address of the user to be added
    /// 
    /// # Authorization
    /// * Requires authorization from the user being added
    pub fn add_user(env: Env, sender: Address, user: Address) {

        sender.require_auth();

        // Check if sender is the user or a manager
        let (is_manager, _) = Self::manager_exists(&env, &sender);
        if sender != user && !is_manager {
            panic!("{:?}", Error::Unauthorized);
        }

        let mut users = env.storage().persistent().get::<DataKey, Map<Address, bool>>(&DataKey::Users).unwrap();
        users.set(user, true);
        env.storage().persistent().set(&DataKey::Users, &users);
    }

    /// Removes a user from the contract's user registry
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `sender` - The address of the account attempting to remove the user
    /// * `user` - The address of the user to be removed
    /// 
    /// # Authorization
    /// * Requires authorization from the sender
    /// * Sender must be either the user themselves or a manager
    /// 
    /// # Panics
    /// * If the sender is neither the user nor a manager
    pub fn remove_user(env: Env, sender: Address, user: Address) {
        sender.require_auth();
        
        // Check if sender is the user or a manager
        let (is_manager, _) = Self::manager_exists(&env, &sender);
        if sender != user && !is_manager {
            panic!("{:?}", Error::Unauthorized);
        }
        
        let mut users = env.storage().persistent().get::<DataKey, Map<Address, bool>>(&DataKey::Users).unwrap();
        users.set(user, false);
        env.storage().persistent().set(&DataKey::Users, &users);
    }

    /// Retrieves the complete map of users and their status
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// 
    /// # Returns
    /// * `Map<Address, bool>` - A map where:
    ///   - Key: User's address
    ///   - Value: User's status (true = active, false = inactive)
    pub fn get_users(env: Env) -> Map<Address, bool> {
        env.storage().persistent().get::<DataKey, Map<Address, bool>>(&DataKey::Users).unwrap()
    }
}

#[cfg(test)]
mod test {
    pub mod old_contract {
        soroban_sdk::contractimport!(
            file = "../../wasm/trustful_stellar_v1_test_upgradable.wasm"
        );
    }
    
    pub mod new_contract {
        soroban_sdk::contractimport!(
            file = "../../wasm/trustful_stellar_v1.wasm"
        );
    } 

    use super::*;
    use soroban_sdk::testutils::Address as _;
    fn setup_contract() -> (Env, Address, ScorerContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();

        // Variables to initialize the contract
        let scorer_creator = Address::generate(&env);
        let mut scorer_badges = Map::new(&env);
        let badge = ScorerBadge {
            name: String::from_str(&env, "Test Badge"),
            issuer: scorer_creator.clone(),
            score: 100,
        };
        scorer_badges.set(1, badge);

        // Register the contract
        let scorer_contract_id = env.register_contract(None, ScorerContract);
        let scorer_client = ScorerContractClient::new(&env, &scorer_contract_id);

        // Initialize contract
        scorer_client.initialize(&scorer_creator, &scorer_badges);

        (env, scorer_creator, scorer_client)
    }

    #[test]
    fn test_initialize() {
        setup_contract();
    }

    #[test]
    #[should_panic(expected = "ContractAlreadyInitialized")]
    fn test_double_initialization() {
        let (env, scorer_creator, client) = setup_contract();
        let scorer_badges = Map::new(&env);
        
        client.initialize(&scorer_creator, &scorer_badges);
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
    fn test_add_user() {
        let (env, scorer_creator, client) = setup_contract();
        let user = Address::generate(&env);
        
        // User can add themselves
        client.add_user(&user, &user);
        
        let users = client.get_users();
        assert!(users.get(user.clone()).unwrap());
    }

    #[test]
    fn test_manager_can_add_user() {
        let (env, scorer_creator, client) = setup_contract();
        let manager = Address::generate(&env);
        let user = Address::generate(&env);
        
        // Add manager first
        client.add_manager(&scorer_creator, &manager);
        
        // Manager can add a user
        client.add_user(&manager, &user);
        
        let users = client.get_users();
        assert!(users.get(user.clone()).unwrap());
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_unauthorized_add_user() {
        let (env, _scorer_creator, client) = setup_contract();
        let unauthorized = Address::generate(&env);
        let user = Address::generate(&env);
        
        // Unauthorized address cannot add users
        client.add_user(&unauthorized, &user);
    }

    #[test]
    fn test_remove_user() {
        let (env, _scorer_creator, client) = setup_contract();
        let user = Address::generate(&env);
        
        // Add user first
        client.add_user(&user, &user);
        
        // User can remove themselves
        client.remove_user(&user, &user);
        
        let users = client.get_users();
        assert!(!users.get(user.clone()).unwrap());
    }

    #[test]
    fn test_manager_can_remove_user() {
        let (env, scorer_creator, client) = setup_contract();
        let manager = Address::generate(&env);
        let user = Address::generate(&env);
        
        // Setup: Add manager and user
        client.add_manager(&scorer_creator, &manager);
        client.add_user(&user, &user);
        
        // Manager can remove user
        client.remove_user(&manager, &user);
        
        let users = client.get_users();
        assert!(!users.get(user.clone()).unwrap());
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_unauthorized_remove_user() {
        let (env, _scorer_creator, client) = setup_contract();
        let unauthorized = Address::generate(&env);
        let user = Address::generate(&env);
        
        // Add user first
        client.add_user(&user, &user);
        
        // Unauthorized address cannot remove users
        client.remove_user(&unauthorized, &user);
    }

    #[test]
    fn test_get_users() {
        let (env, _scorer_creator, client) = setup_contract();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        
        // Add two users
        client.add_user(&user1, &user1);
        client.add_user(&user2, &user2);
        
        let users = client.get_users();
        assert!(users.get(user1.clone()).unwrap());
        assert!(users.get(user2.clone()).unwrap());
    }
}   