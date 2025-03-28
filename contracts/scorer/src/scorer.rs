#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, String, Vec};

// Event topics
const TOPIC_USER: &str = "user";
const TOPIC_MANAGER: &str = "manager";
const TOPIC_UPGRADE: &str = "upgrade";
const TOPIC_INIT: &str = "init";
const TOPIC_BADGE: &str = "badge";

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BadgeId {
    pub name: String,
    pub issuer: Address,
}

#[contracttype]
enum DataKey {
    ScorerCreator,
    ScorerBadges,
    Users,
    Managers,
    Initialized,
    Name,
    Description,
    Icon
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
    ManagersNotFound,
    ScorerCreatorDoesNotExist,
    UserAlreadyExist,
    UserDoesNotExist,
    BadgeAlreadyExists,
    BadgeNotFound,
    InvalidScoreRange,
    EmptyArg,
    ScorerCreatorNotFound,
}

#[contractimpl]
impl ScorerContract {
    /// Contract constructor
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `scorer_creator` - The address of the contract creator who will be the initial manager
    /// * `scorer_badges` - The initial set of badges for the contract
    /// * `name` - The name of the scorer
    /// * `description` - The description of the scorer
    /// * `icon` - The icon URL or identifier for the scorer
    /// 
    /// # Panics
    /// * When the contract is already initialized
    /// * When any of the required string arguments are empty
    /// * When the scorer_creator fails authentication
    pub fn initialize(env: Env, scorer_creator: Address, scorer_badges: Map<BadgeId, u32>, name: String, description: String, icon: String) {
        if name.is_empty() || description.is_empty() || icon.is_empty() {
            panic!("{:?}", Error::EmptyArg);
        }

        // Ensure that the contract is not initialized
        if Self::is_initialized(&env) {
            panic!("{:?}", Error::ContractAlreadyInitialized);
        }

        // Ensure that the scorer creator is the sender
        scorer_creator.require_auth();

        // Create initial managers list with scorer_creator
        let mut initial_managers = Vec::<Address>::new(&env);
        initial_managers.push_back(scorer_creator.clone());

        // Store initial state
        env.storage().persistent().set(&DataKey::ScorerCreator, &scorer_creator);
        env.storage().persistent().set(&DataKey::ScorerBadges, &scorer_badges);
        env.storage().persistent().set(&DataKey::Users, &Map::<Address, bool>::new(&env));
        env.storage().persistent().set(&DataKey::Managers, &initial_managers);
        env.storage().persistent().set(&DataKey::Initialized, &true);
        env.storage().persistent().set(&DataKey::Name, &name);
        env.storage().persistent().set(&DataKey::Description, &description);
        env.storage().persistent().set(&DataKey::Icon, &icon);

        // Emit a initialization event
        env.events().publish(
            (TOPIC_INIT, symbol_short!("contract")),
            (scorer_creator, initial_managers, scorer_badges, name, description, icon),
        );
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
    /// * If the admin address cannot be found in storage
    pub fn upgrade(env: Env, new_wasm_hash: BytesN<32>) {
        let admin: Address = env.storage()
            .persistent()
            .get(&DataKey::ScorerCreator)
            .unwrap_or_else(|| panic!("{:?}", Error::ScorerCreatorNotFound));
        
        admin.require_auth();
        
        // Emit event before upgrade
        env.events().publish(
            (TOPIC_UPGRADE, symbol_short!("wasm")),
            new_wasm_hash.clone(),
        );
        
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

    /// Helper function to check if an address is the contract owner
    /// 
    /// # Arguments
    /// * `env` - The environment object
    /// * `address` - The address to check
    /// 
    /// # Returns
    /// * `bool` - True if the address is the contract owner
    fn is_owner(env: &Env, address: &Address) -> bool {
        let owner = env.storage()
            .persistent()
            .get::<DataKey, Address>(&DataKey::ScorerCreator)
            .unwrap_or_else(|| panic!("{:?}", Error::ScorerCreatorNotFound));
        
        &owner == address
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
        let managers = env.storage()
            .persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::Managers)
            .unwrap_or_else(|| Vec::new(env));
        
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
    /// * If the sender is not the scorer creator (`Error::Unauthorized`)
    /// * If the manager already exists (`Error::ManagerAlreadyExists`)
    pub fn add_manager(env: Env, sender: Address, new_manager: Address) {
        sender.require_auth();
        
        if !Self::is_owner(&env, &sender) {
            panic!("{:?}", Error::Unauthorized);
        }

        let (exists, mut managers) = Self::manager_exists(&env, &new_manager);
        if exists {
            panic!("{:?}", Error::ManagerAlreadyExists);
        }
        
        managers.push_back(new_manager.clone());
        env.storage().persistent().set(&DataKey::Managers, &managers);

        // Emit event for manager addition
        env.events().publish(
            (TOPIC_MANAGER, symbol_short!("add")),
            (sender, new_manager),
        );
    }

    /// Removes a manager from the contract
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `sender` - The address of the account attempting to remove the manager
    /// * `manager_to_remove` - The address of the manager to be removed
    /// 
    /// # Panics
    /// * If the sender is not the scorer creator (`Error::Unauthorized`)
    /// * If the manager does not exist (`Error::ManagerNotFound`)
    pub fn remove_manager(env: Env, sender: Address, manager_to_remove: Address) {
        sender.require_auth();
        
        if !Self::is_owner(&env, &sender) {
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

        // Emit event for manager removal
        env.events().publish(
            (TOPIC_MANAGER, symbol_short!("remove")),
            (sender, manager_to_remove),
        );
    }

    /// Adds a new user to the contract's user registry
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `user` - The address of the user to be added
    /// 
    /// # Authorization
    /// * Requires authorization from the user being added
    /// 
    /// # Panics
    /// * If the user already exists and is active (`Error::UserAlreadyExist`)
    pub fn add_user(env: Env, user: Address) {
        user.require_auth();

        let mut users = env.storage()
            .persistent()
            .get::<DataKey, Map<Address, bool>>(&DataKey::Users)
            .unwrap_or_else(|| Map::new(&env));

        // Check if user already exists and is active
        if users.contains_key(user.clone()) && users.get(user.clone()).unwrap() {
            panic!("{:?}", Error::UserAlreadyExist);
        }

        users.set(user.clone(), true);
        env.storage().persistent().set(&DataKey::Users, &users);

        // Emit event for user addition
        env.events().publish(
            (TOPIC_USER, symbol_short!("add")),
            user,
        );
    }

    /// Removes a user from the contract's user registry
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `user` - The address of the user to be removed
    /// 
    /// # Authorization
    /// * Requires authorization from the user
    /// 
    /// # Panics
    /// * If the user does not exist or is already inactive (`Error::UserDoesNotExist`)
    pub fn remove_user(env: Env, user: Address) {
        user.require_auth();
        
        let mut users = env.storage()
            .persistent()
            .get::<DataKey, Map<Address, bool>>(&DataKey::Users)
            .unwrap_or_else(|| panic!("{:?}", Error::UserDoesNotExist));

        // Check if user doesn't exist or is already inactive
        if !users.contains_key(user.clone()) || !users.get(user.clone()).unwrap() {
            panic!("{:?}", Error::UserDoesNotExist);
        }
        
        users.set(user.clone(), false);
        env.storage().persistent().set(&DataKey::Users, &users);

        // Emit event for user removal
        env.events().publish(
            (TOPIC_USER, symbol_short!("remove")),
            user,
        );
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
        env.storage()
            .persistent()
            .get::<DataKey, Map<Address, bool>>(&DataKey::Users)
            .unwrap_or_else(|| Map::new(&env))
    }

    /// Retrieves all scorer badges from the contract's storage
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// 
    /// # Returns
    /// * `Map<BadgeId, u32>` - A map where:
    ///   - Key: Badge ID (BadgeId struct)
    ///   - Value: Badge score value
    pub fn get_badges(env: Env) -> Map<BadgeId, u32> {
        env.storage()
            .persistent()
            .get::<DataKey, Map<BadgeId, u32>>(&DataKey::ScorerBadges)
            .unwrap_or_else(|| Map::new(&env))
    }

    /// Retrieves all the managers from the contract.
    ///
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    ///
    /// # Returns
    /// * `Vec<Address>` - A vector of all manager addresses
    ///
    /// # Panics
    /// * When the managers vector cannot be found in storage (`Error::ManagersNotFound`)
    pub fn get_managers(env: Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::Managers)
            .unwrap_or_else(|| panic!("{:?}", Error::ManagersNotFound))
    }

    /// Retrieves the address of the contract creator.
    ///
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    ///
    /// # Returns
    /// * `Address` - The address of the scorer creator
    ///
    /// # Panics
    /// * When the creator's address is not found in storage (`Error::ScorerCreatorDoesNotExist`)
    pub fn get_contract_owner(env: Env) -> Address {
        env.storage()
            .persistent()
            .get::<DataKey, Address>(&DataKey::ScorerCreator)
            .unwrap_or_else(|| panic!("{:?}", Error::ScorerCreatorDoesNotExist))
    }

    /// Adds a new badge to the contract
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `sender` - The address of the account attempting to add the badge
    /// * `name` - The name of the badge
    /// * `issuer` - The issuer of the badge
    /// * `score` - The score value of the badge
    /// 
    /// # Panics
    /// * If the sender is not a manager (`Error::Unauthorized`)
    /// * If a badge with the given name and issuer already exists (`Error::BadgeAlreadyExists`)
    /// * If the badge name is empty (`Error::EmptyArg`)
    /// * If the badge score is invalid (greater than 10000) (`Error::InvalidScoreRange`)
    pub fn add_badge(env: Env, sender: Address, name: String, issuer: Address, score: u32) {
        sender.require_auth();
        
        // Check if sender is a manager
        let (is_manager, _) = Self::manager_exists(&env, &sender);
        if !is_manager {
            panic!("{:?}", Error::Unauthorized);
        }
        
        // Validate badge name
        if name.is_empty() {
            panic!("{:?}", Error::EmptyArg);
        }
        
        // Validate badge score
        if score > 10000 {
            panic!("{:?}", Error::InvalidScoreRange);
        }
        
        let mut badges = env.storage()
            .persistent()
            .get::<DataKey, Map<BadgeId, u32>>(&DataKey::ScorerBadges)
            .unwrap_or_else(|| Map::new(&env));
        
        // Create the badge ID and details
        let badge_id = BadgeId {
            name: name.clone(),
            issuer: issuer.clone(),
        };
        
        // Check if badge with this ID already exists
        if badges.contains_key(badge_id.clone()) {
            panic!("{:?}", Error::BadgeAlreadyExists);
        }
        
        badges.set(badge_id.clone(), score.clone());
        env.storage().persistent().set(&DataKey::ScorerBadges, &badges);
        
        env.events().publish(
            (TOPIC_BADGE, symbol_short!("add")),
            (badge_id, score, sender),
        );
    }

    /// Removes a badge from the contract
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// * `sender` - The address of the account attempting to remove the badge
    /// * `name` - The name of the badge to remove
    /// * `issuer` - The issuer of the badge to remove
    /// 
    /// # Panics
    /// * If the sender is not a manager (`Error::Unauthorized`)
    /// * If the badge with the given name and issuer doesn't exist (`Error::BadgeNotFound`)
    pub fn remove_badge(env: Env, sender: Address, name: String, issuer: Address) {
        sender.require_auth();
        
        // Check if sender is a manager
        let (is_manager, _) = Self::manager_exists(&env, &sender);
        if !is_manager {
            panic!("{:?}", Error::Unauthorized);
        }
        
        let mut badges = env.storage()
            .persistent()
            .get::<DataKey, Map<BadgeId, u32>>(&DataKey::ScorerBadges)
            .unwrap_or_else(|| panic!("{:?}", Error::BadgeNotFound));
        
        // Create the badge key
        let badge_id = BadgeId {
            name,
            issuer,
        };
        
        // Check if badge exists
        if !badges.contains_key(badge_id.clone()) {
            panic!("{:?}", Error::BadgeNotFound);
        }
        
        let badge_details = badges.get(badge_id.clone()).unwrap();
        
        badges.remove(badge_id.clone());
        env.storage().persistent().set(&DataKey::ScorerBadges, &badges);
        
        env.events().publish(
            (TOPIC_BADGE, symbol_short!("remove")),
            (badge_id, badge_details, sender),
        );
    }

    /// Retrieves contract metadata (name, description, icon)
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// 
    /// # Returns
    /// * `(String, String, String)` - A tuple containing:
    ///   - name: Contract name
    ///   - description: Contract description
    ///   - icon: Contract icon
    pub fn get_metadata(env: Env) -> (String, String, String) {
        let name = env.storage()
            .persistent()
            .get::<DataKey, String>(&DataKey::Name)
            .unwrap_or_else(|| String::from_str(&env, ""));
            
        let description = env.storage()
            .persistent()
            .get::<DataKey, String>(&DataKey::Description)
            .unwrap_or_else(|| String::from_str(&env, ""));
            
        let icon = env.storage()
            .persistent()
            .get::<DataKey, String>(&DataKey::Icon)
            .unwrap_or_else(|| String::from_str(&env, ""));
            
        (name, description, icon)
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
    use soroban_sdk::testutils::{Address as _, Events};
    use soroban_sdk::IntoVal;

    fn setup_contract() -> (Env, Address, ScorerContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();

        // Variables to initialize the contract
        let scorer_creator = Address::generate(&env);
        
        let badge_id = BadgeId {
            name: String::from_str(&env, "Test Badge"),
            issuer: scorer_creator.clone(),
        };
        
        let mut scorer_badges = Map::<BadgeId, u32>::new(&env);
        scorer_badges.set(badge_id, 100);

        // Register the contract
        let scorer_contract_id = env.register_contract(None, ScorerContract);
        let scorer_client = ScorerContractClient::new(&env, &scorer_contract_id);

        // Initialize contract
        scorer_client.initialize(&scorer_creator, &scorer_badges, &String::from_str(&env, "New_contract"), &String::from_str(&env,"Contract's description."), &String::from_str(&env,"icon.png"));

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
        
        client.initialize(&scorer_creator, &scorer_badges, &String::from_str(&env, "New_contract"), &String::from_str(&env,"Contract's description."),&String::from_str(&env,"icon.png"));
    }

    #[test]
    fn test_add_manager() {
        let (env, scorer_creator, client) = setup_contract();
        let new_manager = Address::generate(&env);
        client.add_manager(&scorer_creator, &new_manager);

        // Verify storage update
        let managers = env.as_contract(&client.address, || {
            env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap()
        });
        assert_eq!(managers, Vec::from_slice(&env, &[scorer_creator.clone(), new_manager.clone()]));

        // Verify event emission - check if the expected event is in the events list
        let expected_event = (
            client.address.clone(),
            (String::from_str(&env, TOPIC_MANAGER), symbol_short!("add")).into_val(&env),
            (scorer_creator, new_manager).into_val(&env)
        );
        
        assert!(env.events().all().contains(&expected_event), 
            "Expected event not found in events list");
    }

    #[test]
    fn test_remove_manager() {
        let (env, scorer_creator, client) = setup_contract();
        let new_manager = Address::generate(&env);

        client.add_manager(&scorer_creator, &new_manager);
        client.remove_manager(&scorer_creator, &new_manager);

        // Verify storage update
        let managers = env.as_contract(&client.address, || {
            env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap()
        });
        assert_eq!(managers, Vec::from_slice(&env, &[scorer_creator.clone()]));

        // Verify event emission - check if the expected event is in the events list
        let expected_event = (
            client.address.clone(),
            (String::from_str(&env, TOPIC_MANAGER), symbol_short!("remove")).into_val(&env),
            (scorer_creator, new_manager).into_val(&env)
        );
        
        assert!(env.events().all().contains(&expected_event), 
            "Remove manager event not found in events list");
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
        
        assert_eq!(managers, Vec::from_slice(&env, &[scorer_creator.clone(), manager1.clone(), manager2.clone(), manager3.clone()]));

        client.remove_manager(&scorer_creator, &manager2);

        let managers_after_remove = env.as_contract(&client.address, || {
            env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap()
        });
        
        assert_eq!(managers_after_remove, Vec::from_slice(&env, &[scorer_creator, manager1, manager3]));
    }

    #[test]
    fn test_upgrade() {
        let (env, _scorer_creator, client) = setup_contract();
        assert_eq!(1, client.contract_version());
        let new_wasm_hash = env.deployer().upload_contract_wasm(old_contract::WASM);
        client.upgrade(&new_wasm_hash);

        // Verify contract version
        assert_eq!(0, client.contract_version());

        // Verify event emission
        let expected_event = (
            client.address.clone(),
            (String::from_str(&env, TOPIC_UPGRADE), symbol_short!("wasm")).into_val(&env),
            new_wasm_hash.into_val(&env)
        );
        
        assert!(env.events().all().contains(&expected_event), 
            "Upgrade event not found in events list");
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
        let (env, _scorer_creator, client) = setup_contract();
        let user = Address::generate(&env);

        client.add_user(&user);
        
        // Verify storage update
        let users = client.get_users();
        assert!(users.get(user.clone()).unwrap());

        // Verify event emission
        let expected_event = (
            client.address.clone(),
            (String::from_str(&env, TOPIC_USER), symbol_short!("add")).into_val(&env),
            user.into_val(&env)
        );
        
        assert!(env.events().all().contains(&expected_event), 
            "Add user event not found in events list");
    }

    #[test]
    fn test_manager_can_add_user() {
        let (env, scorer_creator, client) = setup_contract();
        let manager = Address::generate(&env);
        let user = Address::generate(&env);
        
        // Add manager first
        client.add_manager(&scorer_creator, &manager);
        
        // User adds themselves
        client.add_user(&user);
        
        let users = client.get_users();
        assert!(users.get(user.clone()).unwrap());
    }

    #[test]
    #[should_panic(expected = "UserAlreadyExist")]
    fn test_unauthorized_add_user() {
        let (env, _scorer_creator, client) = setup_contract();
        let user = Address::generate(&env);
        
        // First add the user
        client.add_user(&user);
        
        // Try to add the same user again - should panic with UserAlreadyExist
        client.add_user(&user);
    }

    #[test]
    fn test_remove_user() {
        let (env, _scorer_creator, client) = setup_contract();
        let user = Address::generate(&env);
        
        client.add_user(&user);
        client.remove_user(&user);
        
        // Verify storage update
        let users = client.get_users();
        assert!(!users.get(user.clone()).unwrap());

        // Verify event emission
        let events = env.events().all();
        
        // Check for add event
        let expected_add_event = (
            client.address.clone(),
            (String::from_str(&env, TOPIC_USER), symbol_short!("add")).into_val(&env),
            user.clone().into_val(&env)
        );
        
        // Check for remove event
        let expected_remove_event = (
            client.address.clone(),
            (String::from_str(&env, TOPIC_USER), symbol_short!("remove")).into_val(&env),
            user.into_val(&env)
        );
        
        assert!(events.contains(&expected_add_event), 
            "Add user event not found in events list");
        assert!(events.contains(&expected_remove_event), 
            "Remove user event not found in events list");
    }

    #[test]
    fn test_manager_can_remove_user() {
        let (env, scorer_creator, client) = setup_contract();
        let manager = Address::generate(&env);
        let user = Address::generate(&env);
        
        // Setup: Add manager and user
        client.add_manager(&scorer_creator, &manager);
        client.add_user(&user);
        
        // Manager can remove user
        client.remove_user(&user);
        
        let users = client.get_users();
        assert!(!users.get(user.clone()).unwrap());
    }

    #[test]
    #[should_panic(expected = "UserDoesNotExist")]
    fn test_unauthorized_remove_user() {
        let (env, _scorer_creator, client) = setup_contract();
        let user = Address::generate(&env);
        
        // Add user first
        client.add_user(&user);
        client.remove_user(&user);

        // Unauthorized address cannot remove users
        client.remove_user(&user);
    }

    #[test]
    fn test_get_users() {
        let (env, _scorer_creator, client) = setup_contract();
        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);
        
        // Add two users
        client.add_user(&user1);
        client.add_user(&user2);
        
        let users = client.get_users();
        assert!(users.get(user1.clone()).unwrap());
        assert!(users.get(user2.clone()).unwrap());
    }

    #[test]
    fn test_get_managers() {
        let (env, scorer_creator, client) = setup_contract();
        let new_manager_1 = Address::generate(&env);
        let new_manager_2 = Address::generate(&env);

        client.add_manager(&scorer_creator, &new_manager_1);
        client.add_manager(&scorer_creator, &new_manager_2);

        // Verify storage update
        let managers = client.get_managers();
        assert_eq!(managers, Vec::from_slice(&env, &[scorer_creator.clone(), new_manager_1, new_manager_2]));
    }

    #[test]
    fn test_get_scorer_creator() {
        let (_, scorer_creator, client) = setup_contract();

        // Verify storage update
        let owner = client.get_contract_owner();
        assert_eq!(owner, scorer_creator);
    }

    #[test]
    fn test_add_badge() {
        let (env, scorer_creator, client) = setup_contract();

        let name = String::from_str(&env, "New Test Badge");
        let issuer = scorer_creator.clone();
        let score = 200;
        
        client.add_badge(&scorer_creator, &name, &issuer, &score);
        
        // Verify the badge was added
        let badges = client.get_badges();
        
        let badge_id = BadgeId {
            name: name.clone(),
            issuer: issuer.clone(),
        };
        
        assert!(badges.contains_key(badge_id.clone()));
        let stored_details = badges.get(badge_id.clone()).unwrap();
        assert_eq!(stored_details, score);
        
        // Verify event emission
        let expected_event = (
            client.address.clone(),
            (String::from_str(&env, TOPIC_BADGE), symbol_short!("add")).into_val(&env),
            (badge_id, stored_details, scorer_creator).into_val(&env)
        );
        
        assert!(env.events().all().contains(&expected_event), 
            "Add badge event not found in events list");
    }

    #[test]
    fn test_remove_badge() {
        let (env, scorer_creator, client) = setup_contract();
        
        // Create a new badge to add and then remove
        let name = String::from_str(&env, "Badge to Remove");
        let issuer = scorer_creator.clone();
        let score = 150;
        
        // Add the badge with the new method
        client.add_badge(&scorer_creator, &name, &issuer, &score);
        
        // Create badge ID for verification
        let badge_id = BadgeId {
            name: name.clone(),
            issuer: issuer.clone(),
        };

        // Remove the badge
        client.remove_badge(&scorer_creator, &name, &issuer);
        
        // Verify the badge was removed
        let badges_after = client.get_badges();
        assert!(!badges_after.contains_key(badge_id.clone()));
        
        // Verify event emission (should have both add and remove events)
        let events = env.events().all();
        
        // Check for add event
        let expected_add_event = (
            client.address.clone(),
            (String::from_str(&env, TOPIC_BADGE), symbol_short!("add")).into_val(&env),
            (badge_id.clone(), score.clone(), scorer_creator.clone()).into_val(&env)
        );
        
        // Check for remove event
        let expected_remove_event = (
            client.address.clone(),
            (String::from_str(&env, TOPIC_BADGE), symbol_short!("remove")).into_val(&env),
            (badge_id, score, scorer_creator).into_val(&env)
        );
        
        // Check if both events exist in the events list
        assert!(events.contains(&expected_add_event), "Add event not found in events list");
        assert!(events.contains(&expected_remove_event), "Remove event not found in events list");
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_add_badge_unauthorized() {
        let (env, _scorer_creator, client) = setup_contract();
        
        // Create an unauthorized user
        let unauthorized_user = Address::generate(&env);
        
        // Create a new badge
        let name = String::from_str(&env, "Unauthorized Badge");
        let issuer = unauthorized_user.clone();
        let score = 50;
        
        // This should panic because unauthorized_user is not a manager
        client.add_badge(&unauthorized_user, &name, &issuer, &score);
    }

    #[test]
    #[should_panic(expected = "BadgeNotFound")]
    fn test_remove_nonexistent_badge() {
        let (env, scorer_creator, client) = setup_contract();
        
        // Try to remove a badge that doesn't exist
        let nonexistent_name = String::from_str(&env, "Nonexistent Badge");
        let issuer = scorer_creator.clone();
        
        client.remove_badge(&scorer_creator, &nonexistent_name, &issuer);
    }

    #[test]
    #[should_panic(expected = "BadgeAlreadyExists")]
    fn test_add_duplicate_badge() {
        let (env, scorer_creator, client) = setup_contract();
        
        // Create a new badge
        let name = String::from_str(&env, "First Badge");
        let issuer = scorer_creator.clone();
        let score = 100;
        
        // Add the badge
        client.add_badge(&scorer_creator, &name, &issuer, &score);
        
        // Try to add the same badge again (same name and issuer)
        client.add_badge(&scorer_creator, &name, &issuer, &300);
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_remove_badge_unauthorized() {
        let (env, scorer_creator, client) = setup_contract();
        
        // Create a new badge
        let name = String::from_str(&env, "Badge");
        let issuer = scorer_creator.clone();
        let score = 100;
        
        // Add the badge
        client.add_badge(&scorer_creator, &name, &issuer, &score);
        
        // Create an unauthorized user
        let unauthorized_user = Address::generate(&env);
        
        // This should panic because unauthorized_user is not a manager
        client.remove_badge(&unauthorized_user, &name, &issuer);
    }

    #[test]
    #[should_panic(expected = "EmptyArg")]
    fn test_add_badge_empty_name() {
        let (env, scorer_creator, client) = setup_contract();
        
        let name = String::from_str(&env, "");
        let issuer = scorer_creator.clone();
        let score = 100;
        
        client.add_badge(&scorer_creator, &name, &issuer, &score);
    }

    #[test]
    fn test_manager_can_add_and_remove_badge() {
        let (env, scorer_creator, client) = setup_contract();
        let manager = Address::generate(&env);
        
        // Add a new manager
        client.add_manager(&scorer_creator, &manager);
        
        // Manager adds a badge
        let name = String::from_str(&env, "Manager Badge");
        let issuer = manager.clone();
        let score = 200;
        
        client.add_badge(&manager, &name, &issuer, &score);
        
        // Create badge ID for verification
        let badge_id = BadgeId {
            name: name.clone(),
            issuer: issuer.clone(),
        };
        
        // Verify the badge was added
        let badges = client.get_badges();
        assert!(badges.contains_key(badge_id.clone()));
        
        // Manager removes the badge
        client.remove_badge(&manager, &name, &issuer);
        
        // Verify the badge was removed
        let badges_after = client.get_badges();
        assert!(!badges_after.contains_key(badge_id));
    }

    #[test]
    fn test_get_contract_version() {
        let (_, _, client) = setup_contract();
        
        // Verify initial contract version
        assert_eq!(1, client.contract_version());
    }

    #[test]
    #[should_panic(expected = "EmptyArg")]
    fn test_initialize_empty_args() {
        let (env, scorer_creator, scorer_client) = setup_contract();
        let scorer_badges = Map::new(&env);
        
        // Teste com nome vazio
        scorer_client.initialize(
            &scorer_creator,
            &scorer_badges,
            &String::from_str(&env, ""),
            &String::from_str(&env, "Description"),
            &String::from_str(&env, "icon.png")
        );
    }

    #[test]
    fn test_add_badge_max_score() {
        let (env, scorer_creator, client) = setup_contract();
        
        let name = String::from_str(&env, "Max Score Badge");
        let issuer = scorer_creator.clone();
        let score = 10000;
        
        client.add_badge(&scorer_creator, &name, &issuer, &score);
        
        let badges = client.get_badges();
        let badge_id = BadgeId {
            name: name.clone(),
            issuer: issuer.clone(),
        };
        
        assert_eq!(badges.get(badge_id).unwrap(), score);
    }

    #[test]
    fn test_initialize_storage_state() {
        let env = Env::default();
        env.mock_all_auths();

        let scorer_creator = Address::generate(&env);
        let scorer_badges = Map::new(&env);
        let name = String::from_str(&env, "Test Name");
        let description = String::from_str(&env, "Test Description");
        let icon = String::from_str(&env, "test_icon.png");
        
        let scorer_contract_id = env.register_contract(None, ScorerContract);
        let client = ScorerContractClient::new(&env, &scorer_contract_id);

        client.initialize(
            &scorer_creator,
            &scorer_badges,
            &name,
            &description,
            &icon
        );

        let is_initialized: bool = env.as_contract(&client.address, || {
            env.storage().persistent().get(&DataKey::Initialized).unwrap()
        });
        assert!(is_initialized);

        let stored_creator = client.get_contract_owner();
        assert_eq!(stored_creator, scorer_creator);

        let managers = client.get_managers();
        assert_eq!(managers.len(), 1);
        assert_eq!(managers.get(0).unwrap(), scorer_creator);

        let expected_init_event = (
            client.address.clone(),
            (String::from_str(&env, TOPIC_INIT), symbol_short!("contract")).into_val(&env),
            (
                scorer_creator,
                managers,
                scorer_badges,
                name,
                description,
                icon
            ).into_val(&env)
        );
        
        assert!(env.events().all().contains(&expected_init_event), 
            "Initialization event not found in events list");
    }

    #[test]
    fn test_get_contract_metadata() {
        let env = Env::default();
        env.mock_all_auths();

        let scorer_creator = Address::generate(&env);
        let scorer_badges = Map::new(&env);
        let name = String::from_str(&env, "Test Name");
        let description = String::from_str(&env, "Test Description");
        let icon = String::from_str(&env, "test_icon.png");
        
        let scorer_contract_id = env.register_contract(None, ScorerContract);
        let client = ScorerContractClient::new(&env, &scorer_contract_id);

        client.initialize(
            &scorer_creator,
            &scorer_badges,
            &name,
            &description,
            &icon
        );

        let stored_name: String = env.as_contract(&client.address, || {
            env.storage().persistent().get(&DataKey::Name).unwrap()
        });
        assert_eq!(stored_name, name);

        let stored_description: String = env.as_contract(&client.address, || {
            env.storage().persistent().get(&DataKey::Description).unwrap()
        });
        assert_eq!(stored_description, description);

        let stored_icon: String = env.as_contract(&client.address, || {
            env.storage().persistent().get(&DataKey::Icon).unwrap()
        });
        assert_eq!(stored_icon, icon);
    }

    #[test]
    fn test_user_read_after_remove() {
        let env = Env::default();
        env.mock_all_auths();

        let scorer_creator = Address::generate(&env);
        let scorer_badges = Map::new(&env);
        let user = Address::generate(&env);
        
        let scorer_contract_id = env.register_contract(None, ScorerContract);
        let client = ScorerContractClient::new(&env, &scorer_contract_id);

        client.initialize(
            &scorer_creator,
            &scorer_badges,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, "Description"),
            &String::from_str(&env, "icon.png")
        );

        client.add_user(&user);
        
        client.remove_user(&user);
        
        client.add_user(&user);
        
        let users = client.get_users();
        assert!(users.get(user).unwrap());
    }

    #[test]
    #[should_panic(expected = "InvalidScoreRange")]
    fn test_add_badge_score_above_max() {
        let env = Env::default();
        env.mock_all_auths();

        let scorer_creator = Address::generate(&env);
        let scorer_badges = Map::new(&env);
        
        let scorer_contract_id = env.register_contract(None, ScorerContract);
        let client = ScorerContractClient::new(&env, &scorer_contract_id);

        client.initialize(
            &scorer_creator,
            &scorer_badges,
            &String::from_str(&env, "Test"),
            &String::from_str(&env, "Description"),
            &String::from_str(&env, "icon.png")
        );
        
        client.add_badge(
            &scorer_creator,
            &String::from_str(&env, "Test Badge"),
            &scorer_creator,
            &10001
        );
    }
}   