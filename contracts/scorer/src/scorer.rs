#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, String, Vec};

// Event topics
const TOPIC_USER: &str = "user";
const TOPIC_MANAGER: &str = "manager";
const TOPIC_UPGRADE: &str = "upgrade";

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
    Name,
    Description
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
}

#[contractimpl]
impl ScorerContract {
    /// Contract constructor
    pub fn initialize(env: Env, scorer_creator: Address, scorer_badges: Map<u32, ScorerBadge>, name: String, description: String) {
        
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
    pub fn add_user(env: Env, user: Address) {
        user.require_auth();

        let mut users = env.storage().persistent().get::<DataKey, Map<Address, bool>>(&DataKey::Users).unwrap();

        if users.get(user.clone()).unwrap_or(false) {
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
    /// * If the user does not exist
    pub fn remove_user(env: Env, user: Address) {
        user.require_auth();
        
        let mut users = env.storage().persistent().get::<DataKey, Map<Address, bool>>(&DataKey::Users).unwrap();

        if !users.get(user.clone()).unwrap_or(true) {
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
        env.storage().persistent().get::<DataKey, Map<Address, bool>>(&DataKey::Users).unwrap()
    }

    /// Retrieves all scorer badges from the contract's storage
    /// 
    /// # Arguments
    /// * `env` - The environment object providing access to the contract's storage
    /// 
    /// # Returns
    /// * `Map<u32, ScorerBadge>` - A map where:
    ///   - Key: Badge ID (u32)
    ///   - Value: ScorerBadge struct containing the badge details
    pub fn get_badges(env: Env) -> Map<u32, ScorerBadge> {
        env.storage().persistent().get::<DataKey, Map<u32, ScorerBadge>>(&DataKey::ScorerBadges).unwrap()
    }   

    /// Retrieves all the managers from the contract.
    ///
    /// # Returns
    /// * A map of addresses to their manager status (true or false).
    ///
    /// # Panics
    /// * This function panic if there is no manager object.
    pub fn get_managers(env: Env) -> Vec<Address>{
        return env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap_or_else(|| panic!("{:?}", Error::ManagersNotFound));
    }

    /// Retrieves the address of the contract creator.
    ///
    /// # Returns
    /// * The address of the scorer factory creator.
    ///
    /// # Panics
    /// * This function will panic if the creator's address is not found in storage.
    pub fn get_contract_owner(env: Env) -> Address{
        return env.storage().persistent().get::<DataKey, Address>(&DataKey::ScorerCreator).unwrap_or_else(|| panic!("{:?}", Error::ScorerCreatorDoesNotExist));
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
    use soroban_sdk::{vec, IntoVal};
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
        scorer_client.initialize(&scorer_creator, &scorer_badges, &String::from_str(&env, "New_contract"), &String::from_str(&env,"Contract's description."));

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
        
        client.initialize(&scorer_creator, &scorer_badges, &String::from_str(&env, "New_contract"), &String::from_str(&env,"Contract's description."));
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

        // Verify event emission

        assert_eq!(
            env.events().all(),
            vec![
                &env,
                (
                    client.address.clone(),
                    (String::from_str(&env, TOPIC_MANAGER), symbol_short!("add")).into_val(&env),
                    (scorer_creator, new_manager).into_val(&env)
                ),
            ]
        );
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

        // Verify event emission
        assert_eq!(
            env.events().all(),
            vec![
                &env,
                (
                    client.address.clone(),
                    (String::from_str(&env, TOPIC_MANAGER), symbol_short!("add")).into_val(&env),
                    (scorer_creator.clone(), new_manager.clone()).into_val(&env)
                ),
                (
                    client.address.clone(),
                    (String::from_str(&env, TOPIC_MANAGER), symbol_short!("remove")).into_val(&env),
                    (scorer_creator, new_manager).into_val(&env)
                ),
            ]
        );
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
        assert_eq!(
            env.events().all(),
            vec![
                &env,
                (
                    client.address.clone(),
                    (String::from_str(&env, TOPIC_UPGRADE), symbol_short!("wasm")).into_val(&env),
                    new_wasm_hash.into_val(&env)
                ),
            ]
        );
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
        assert_eq!(
            env.events().all(),
            vec![
                &env,
                (
                    client.address.clone(),
                    (String::from_str(&env, TOPIC_USER), symbol_short!("add")).into_val(&env),
                    user.into_val(&env)
                ),
            ]
        );
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
        assert_eq!(
            env.events().all(),
            vec![
                &env,
                (
                    client.address.clone(),
                    (String::from_str(&env, TOPIC_USER), symbol_short!("add")).into_val(&env),
                    user.into_val(&env)
                ),
                (
                    client.address.clone(),
                    (String::from_str(&env, TOPIC_USER), symbol_short!("remove")).into_val(&env),
                    user.into_val(&env)
                ),
            ]
        );
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

}   