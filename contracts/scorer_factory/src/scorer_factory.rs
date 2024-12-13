#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Map, Address, Env};

#[contracttype]
enum DataKey {
    CreatedScorers,
    Initialized,
    ScorerFactoryCreator,
    Managers,
}

#[contracttype]
#[derive(Debug)]
enum Error {
    ContractAlreadyInitialized,
    Unauthorized,
    ManagerAlreadyExists,
    ManagerNotFound,
}

#[contract]
pub struct ScorerFactoryContract;

#[contractimpl]
impl ScorerFactoryContract {
    
    pub fn initialize(env: Env, scorer_creator: Address) {
        if Self::is_initialized(env.clone()) {
            panic!("{:?}", Error::ContractAlreadyInitialized);
        }
        scorer_creator.require_auth();

        let mut managers = Map::<Address, bool>::new(&env);
        managers.set(scorer_creator.clone(), true);
        
        env.storage().persistent().set(&DataKey::Initialized, &true);
        env.storage().persistent().set(&DataKey::ScorerFactoryCreator, &scorer_creator);
        env.storage().persistent().set(&DataKey::Managers, &managers);
    }

    pub fn is_initialized(env: Env) -> bool {
        env.storage().persistent().get::<DataKey, bool>(&DataKey::Initialized).unwrap_or(false)
    }

    pub fn is_scorer_factory_creator(env: Env, address: Address) -> bool {
        env.storage().persistent().get::<DataKey, Address>(&DataKey::ScorerFactoryCreator).unwrap() == address
    }  
    pub fn is_manager(env: Env, address: Address) -> bool {
        env.storage().persistent().get::<DataKey, Map<Address, bool>>(&DataKey::Managers).unwrap().get(address).unwrap_or(false)
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    fn setup_contract() -> (Env, Address, ScorerFactoryContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let scorer_factory_creator = Address::generate(&env);
        let scorer_factory_contract_id = env.register_contract(None, ScorerFactoryContract);
        let scorer_factory_client: ScorerFactoryContractClient<'_> = ScorerFactoryContractClient::new(&env, &scorer_factory_contract_id);
        scorer_factory_client.initialize(&scorer_factory_creator);
        (env, scorer_factory_creator, scorer_factory_client)
    }

    #[test]
    fn test_initialize() {
        let (env, scorer_factory_creator, scorer_factory_client) = setup_contract();
        assert!(scorer_factory_client.is_initialized());
        assert!(scorer_factory_client.is_scorer_factory_creator(&scorer_factory_creator));
    }

    #[test]
    fn test_is_manager() {
        let (env, scorer_factory_creator, scorer_factory_client) = setup_contract();
        assert!(scorer_factory_client.is_manager(&scorer_factory_creator));
    }
}