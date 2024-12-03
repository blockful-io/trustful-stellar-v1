use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String};

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
        env.storage().instance().set(&"scorer_creator", &scorer_creator);
        env.storage().instance().set(&"scorer_badges", &scorer_badges);
        env.storage().instance().set(&"user_scores", &Map::<Address, u32>::new(&env));
        env.storage().instance().set(&"initialized", &true);
    }

    // Helper function to check initialization
    fn is_initialized(env: &Env) -> bool {
        env.storage().instance().get(&"initialized").unwrap_or(false)
    }
   
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    
    #[test]
    fn test_initialize() {
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
        scorer_client.initialize(&scorer_creator.clone(), &scorer_badges);
    }
}   