use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScorerBadge {
    pub name: String,
    pub issuer: Address,
    pub score: u32,
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
        env.storage().persistent().set(&"scorer_creator", &scorer_creator);
        env.storage().persistent().set(&"scorer_badges", &scorer_badges);
        env.storage().persistent().set(&"user_scores", &Map::<Address, u32>::new(&env));
        env.storage().persistent().set(&"managers", &Vec::<Address>::new(&env));
        env.storage().persistent().set(&"initialized", &true);
    }

    // Helper function to check initialization
    fn is_initialized(env: &Env) -> bool {
        env.storage().persistent().get(&"initialized").unwrap_or(false)
    }

    // Helper function to check if a manager exists
    fn manager_exists(managers: &Vec<Address>, manager: Address) -> bool {
        managers.iter().any(|m| m == manager)
    }

    pub fn add_manager(env: Env, sender: Address, new_manager: Address) {
        sender.require_auth();
        
        if sender != env.storage().persistent().get::<&str, Address>(&"scorer_creator").unwrap() {
            panic!("Only the scorer creator can add managers");
        }

        let mut managers = env.storage().persistent().get::<&str, Vec<Address>>(&"managers").unwrap();
        if Self::manager_exists(&managers, new_manager.clone()) {
            panic!("Manager already exists");
        }
        
        managers.push_back(new_manager);
        env.storage().persistent().set(&"managers", &managers);
    }

    pub fn remove_manager(env: Env, sender: Address, manager_to_remove: Address) {
        sender.require_auth();
        
        if sender != env.storage().persistent().get::<&str, Address>(&"scorer_creator").unwrap() {
            panic!("Only the scorer creator can remove managers");
        }
        
        let mut managers = env.storage().persistent().get::<&str, Vec<Address>>(&"managers").unwrap();
        
        if !Self::manager_exists(&managers, manager_to_remove.clone()) {
            panic!("Manager not found");
        }
        
        if let Some(index) = managers.iter().position(|m| m == manager_to_remove) {
            managers.remove(index as u32);
            env.storage().persistent().set(&"managers", &managers);
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
            env.storage().persistent().get::<&str, Vec<Address>>(&"managers").unwrap()
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
            env.storage().persistent().get::<&str, Vec<Address>>(&"managers").unwrap()
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
            env.storage().persistent().get::<&str, Vec<Address>>(&"managers").unwrap()
        });
        
        assert_eq!(managers, Vec::from_slice(&env, &[manager1.clone(), manager2.clone(), manager3.clone()]));

        client.remove_manager(&scorer_creator, &manager2);

        let managers_after_remove = env.as_contract(&client.address, || {
            env.storage().persistent().get::<&str, Vec<Address>>(&"managers").unwrap()
        });
        
        assert_eq!(managers_after_remove, Vec::from_slice(&env, &[manager1, manager3]));
    }
}   