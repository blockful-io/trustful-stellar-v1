#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Map, Address, Env, BytesN, Symbol, Val, Vec, symbol_short, String};
// use scorer::ScorerBadge;

// Event topics
const TOPIC_SCORER: &str = "scorer";
const TOPIC_MANAGER: &str = "manager"; 

#[contracttype]
enum DataKey {
    CreatedScorers,
    Initialized,
    ScorerFactoryCreator,
    Managers,
    ScorerWasmHash,
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
    
    /// Initializes the ScorerFactory contract with the initial manager (scorer_creator)
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `scorer_creator` - The address that will be set as both the factory creator and initial manager
    /// * `scorer_wasm_hash` - The hash of the Wasm binary for the scorer contract
    /// 
    /// # Panics
    /// * When the contract is already initialized
    /// * When the scorer_creator fails authentication
    pub fn initialize(env: Env, scorer_creator: Address, scorer_wasm_hash: BytesN<32>) {
        if Self::is_initialized(env.clone()) {
            panic!("{:?}", Error::ContractAlreadyInitialized);
        }
        scorer_creator.require_auth();

        let mut managers = Map::<Address, bool>::new(&env);
        managers.set(scorer_creator.clone(), true);
        
        env.storage().persistent().set(&DataKey::Initialized, &true);
        env.storage().persistent().set(&DataKey::ScorerFactoryCreator, &scorer_creator);
        env.storage().persistent().set(&DataKey::Managers, &managers);
        env.storage().persistent().set(&DataKey::ScorerWasmHash, &scorer_wasm_hash);
    }

    /// Checks if the contract has been initialized
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// 
    /// # Returns
    /// * `bool` - True if the contract is initialized, false otherwise
    pub fn is_initialized(env: Env) -> bool {
        env.storage().persistent().get::<DataKey, bool>(&DataKey::Initialized).unwrap_or(false)
    }

    /// Verifies if the provided address is the scorer factory creator
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - The address to check
    /// 
    /// # Returns
    /// * `bool` - True if the address is the scorer factory creator, false otherwise
    pub fn is_scorer_factory_creator(env: Env, address: Address) -> bool {
        env.storage().persistent().get::<DataKey, Address>(&DataKey::ScorerFactoryCreator).unwrap() == address
    }  
    /// Checks if the provided address is a manager
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - The address to check
    /// 
    /// # Returns
    /// * `bool` - True if the address is a manager, false otherwise
    pub fn is_manager(env: Env, address: Address) -> bool {
        env.storage().persistent().get::<DataKey, Map<Address, bool>>(&DataKey::Managers).unwrap().get(address).unwrap_or(false)
    }

    /// Deploy a new scorer contract
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `deployer` - The address that will deploy the scorer contract
    /// * `salt` - A unique value to ensure unique contract addresses
    /// * `init_fn` - The initialization function name to call on the deployed contract
    /// * `init_args` - Arguments to pass to the initialization function
    /// 
    /// # Returns
    /// * `Address` - The address of the newly deployed scorer contract
    /// 
    /// # Panics
    /// * When the deployer is not the current contract and fails authentication
    /// * When the deployer is not a registered manager (`Error::Unauthorized`)
    pub fn create_scorer(
        env: Env,
        deployer: Address,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
    ) -> Address {
        // Skip authorization if deployer is the current contract
        if deployer != env.current_contract_address() {
            deployer.require_auth();
        }

        // Verify deployer is a manager
        if !Self::is_manager(env.clone(), deployer.clone()) {
            panic!("{:?}", Error::Unauthorized);
        }

        // Get the stored WASM hash
        let wasm_hash = env.storage()
            .persistent()
            .get::<DataKey, BytesN<32>>(&DataKey::ScorerWasmHash)
            .unwrap();

        // Deploy the contract using the stored Wasm hash
        let scorer_address = env
            .deployer()
            .with_address(deployer.clone(), salt)
            .deploy(wasm_hash);

        // Initialize the contract
        let _: () = env.invoke_contract(&scorer_address, &init_fn, init_args);
        
        // Record the created scorer
        let mut created_scorers = env.storage()
            .persistent()
            .get::<DataKey, Map<Address, bool>>(&DataKey::CreatedScorers)
            .unwrap_or_else(|| Map::new(&env));
        created_scorers.set(scorer_address.clone(), true);
        env.storage().persistent().set(&DataKey::CreatedScorers, &created_scorers);

        env.events().publish((TOPIC_SCORER, symbol_short!("create")), (deployer, scorer_address.clone()));

        scorer_address
    }

    /// Returns a map of all scorer contracts created by this factory
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// 
    /// # Returns
    /// * `Map<Address, bool>` - A map where keys are scorer contract addresses and values are always true
    pub fn get_scorers(env: Env) -> Map<Address, bool> {
        env.storage().persistent().get::<DataKey, Map<Address, bool>>(&DataKey::CreatedScorers).unwrap_or(Map::new(&env))
    }

    /// Adds a new manager to the contract
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `caller` - The address that will authenticate the addition of the new manager
    /// * `manager` - The address to be added as a manager
    /// 
    /// # Panics
    /// * When the caller is not the scorer factory creator or a manager
    pub fn add_manager(env: Env, caller: Address, manager: Address) {
        // Require authentication from the caller
        caller.require_auth();

        // Verify caller is factory creator or a manager
        if !Self::is_scorer_factory_creator(env.clone(), caller.clone()) 
            && !Self::is_manager(env.clone(), caller.clone()) {
            panic!("{:?}", Error::Unauthorized);
        }

        let mut managers = env.storage().persistent()
            .get::<DataKey, Map<Address, bool>>(&DataKey::Managers)
            .unwrap_or(Map::new(&env));
        managers.set(manager.clone(), true);
        env.storage().persistent().set(&DataKey::Managers, &managers);

        env.events().publish((TOPIC_MANAGER, symbol_short!("add")), (caller, manager));
    }
    
    /// Removes a manager from the contract by setting their status to false
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `caller` - The address that will authenticate the removal of the manager
    /// * `manager` - The address to be removed as a manager
    /// 
    /// # Panics
    /// * When the caller is not the scorer factory creator or a manager
    pub fn remove_manager(env: Env, caller: Address, manager: Address) {
        // Require authentication from the caller
        caller.require_auth();

        // Verify caller is factory creator or a manager
        if !Self::is_scorer_factory_creator(env.clone(), caller.clone()) 
            && !Self::is_manager(env.clone(), caller.clone()) {
            panic!("{:?}", Error::Unauthorized);
        }

        let mut managers = env.storage().persistent()
            .get::<DataKey, Map<Address, bool>>(&DataKey::Managers)
            .unwrap_or(Map::new(&env));
        managers.set(manager.clone(), false);
        env.storage().persistent().set(&DataKey::Managers, &managers);
        env.events().publish((TOPIC_MANAGER, symbol_short!("remove")), (caller, manager));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::Address as _, IntoVal};
    
    fn install_scorer_wasm(e: &Env) -> BytesN<32> {
        soroban_sdk::contractimport!(
            file = "../../wasm/scorer.wasm"
        );
        e.deployer().upload_contract_wasm(WASM)
    }

    fn setup_contract() -> (Env, Address, ScorerFactoryContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let scorer_factory_creator = Address::generate(&env);
        let scorer_factory_contract_id = env.register_contract(None, ScorerFactoryContract);
        let scorer_factory_client = ScorerFactoryContractClient::new(&env, &scorer_factory_contract_id);
        
        // Upload the real scorer WASM and get its hash
        let wasm_hash = install_scorer_wasm(&env);
        
        scorer_factory_client.initialize(&scorer_factory_creator, &wasm_hash);
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

    #[test]
    fn test_get_scorers() {
        let (env, scorer_factory_creator, scorer_factory_client) = setup_contract();
        let scorers = scorer_factory_client.get_scorers();
        assert!(scorers.len() == 0);
    }
}