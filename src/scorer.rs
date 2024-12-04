use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String, Vec};

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
    UserScores,
    Managers,
    Initialized,
}

#[contract]
pub struct ScorerContract;



#[contractimpl]
impl ScorerContract {
    /// Contract constructor
    pub fn initialize(env: Env, scorer_creator: Address, scorer_badges: Map<u32, ScorerBadge>) {
        
        // Ensure that the contract is not initialized
        if Self::is_initialized(&env) {
            panic!("Contract already initialized");
        }

        // Ensure that the scorer creator is the sender
        scorer_creator.require_auth();

        // Store initial state
        env.storage().persistent().set(&DataKey::ScorerCreator, &scorer_creator);
        env.storage().persistent().set(&DataKey::ScorerBadges, &scorer_badges);
        env.storage().persistent().set(&DataKey::UserScores, &Map::<Address, u32>::new(&env));
        env.storage().persistent().set(&DataKey::Managers, &Vec::<Address>::new(&env));
        env.storage().persistent().set(&DataKey::Initialized, &true);
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
            panic!("Only the scorer creator can add managers");
        }

        let (exists, mut managers) = Self::manager_exists(&env, &new_manager);
        if exists {
            panic!("Manager already exists");
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
            panic!("Only the scorer creator can remove managers");
        }
        
        let (exists, mut managers) = Self::manager_exists(&env, &manager_to_remove);
        if !exists {
            panic!("Manager not found");
        }
        
        if let Some(index) = managers.iter().position(|m| m == manager_to_remove) {
            managers.remove(index as u32);
            env.storage().persistent().set(&DataKey::Managers, &managers);
        }
    }
}

#[cfg(test)]
mod test {
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
    #[should_panic(expected = "Contract already initialized")]
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
    #[should_panic(expected = "Only the scorer creator can add managers")]
    fn test_add_manager_unauthorized() {
        let (env, _scorer_creator, client) = setup_contract();
        let unauthorized_user = Address::generate(&env);
        let new_manager = Address::generate(&env);
        
        client.add_manager(&unauthorized_user, &new_manager);
    }

    #[test]
    #[should_panic(expected = "Only the scorer creator can remove managers")]
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
}   