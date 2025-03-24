#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, String, Symbol, Val, Vec, FromVal};
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
    ManagersNotFound,
    ContractCreatorNotFound,
    ScorersWereNotFound,
    ScorerNotFound
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

        let mut managers = Vec::<Address>::new(&env);
        managers.push_back(scorer_creator.clone());
        
        env.storage().persistent().set(&DataKey::Initialized, &true);
        env.storage().persistent().set(&DataKey::ScorerFactoryCreator, &scorer_creator);
        env.storage().persistent().set(&DataKey::Managers, &managers);
        env.storage().persistent().set(&DataKey::ScorerWasmHash, &scorer_wasm_hash);
        env.storage().persistent().set(&DataKey::CreatedScorers, &Map::<Address, (String, String, String)>::new(&env));
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
        return env.storage().persistent().get::<DataKey, Vec<Address>>(&DataKey::Managers).unwrap().contains(address);
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
        let _: () = env.invoke_contract(&scorer_address, &init_fn, init_args.clone());
        
        // Record the created scorer
        let mut created_scorers = env.storage()
            .persistent()
            .get::<DataKey, Map<Address, (String, String, String)>>(&DataKey::CreatedScorers)
            .unwrap_or_else(|| Map::new(&env));

        // Extract name, description ans icon from init_args 
        let args_len = init_args.len();
        let scorer_icon = String::from_val(&env, &init_args.get(args_len - 1).unwrap());
        let scorer_description = String::from_val(&env, &init_args.get(args_len - 2).unwrap());
        let scorer_name = String::from_val(&env, &init_args.get(args_len - 3).unwrap());
            
        created_scorers.set(scorer_address.clone(), (scorer_name.clone(), scorer_description.clone(), scorer_icon.clone()));
        env.storage().persistent().set(&DataKey::CreatedScorers, &created_scorers);
        env.events().publish((TOPIC_SCORER, symbol_short!("create")), (deployer, scorer_address.clone(), scorer_name, scorer_description, scorer_icon));

        scorer_address
    }

    /// Returns a map of all scorer contracts created by this factory
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// 
    /// # Returns
    /// * `Map<Address, bool>` - A map where keys are scorer contract addresses and values are always true
    pub fn get_scorers(env: Env) -> Map<Address, (String, String, String)> {
        return env.storage().persistent().get::<DataKey, Map<Address, (String, String, String)>>(&DataKey::CreatedScorers).unwrap_or_else(|| panic!("{:?}", Error::ScorersWereNotFound));
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
    /// * When the manager already exists
    pub fn add_manager(env: Env, caller: Address, manager: Address) {
        // Require authentication from the caller
        caller.require_auth();

        // Verify caller is factory creator or a manager
        if !Self::is_scorer_factory_creator(env.clone(), caller.clone()) 
            && !Self::is_manager(env.clone(), caller.clone()) {
            panic!("{:?}", Error::Unauthorized);
        }

        let mut managers = env.storage().persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::Managers)
            .unwrap_or(Vec::new(&env));
        
        // Check if manager already exists to avoid duplication
        if managers.contains(manager.clone()) {
            panic!("{:?}", Error::ManagerAlreadyExists);
        }
        
        managers.push_back(manager.clone());
        env.storage().persistent().set(&DataKey::Managers, &managers);

        env.events().publish((TOPIC_MANAGER, symbol_short!("add")), (caller, manager));
    }
    
    /// Removes a manager from the contract
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `caller` - The address that will authenticate the removal of the manager
    /// * `manager` - The address to be removed as a manager
    /// 
    /// # Panics
    /// * When the caller is not the scorer factory creator or a manager
    /// * When the manager to be removed is not found
    pub fn remove_manager(env: Env, caller: Address, manager: Address) {
        // Require authentication from the caller
        caller.require_auth();

        // Verify caller is factory creator or a manager
        if !Self::is_scorer_factory_creator(env.clone(), caller.clone()) 
            && !Self::is_manager(env.clone(), caller.clone()) {
            panic!("{:?}", Error::Unauthorized);
        }

        let mut managers = env.storage().persistent()
            .get::<DataKey, Vec<Address>>(&DataKey::Managers)
            .unwrap_or(Vec::new(&env));

        let mut index_to_remove: Option<u32> = None;
        for i in 0..managers.len() {
            if managers.get(i).unwrap() == manager {
                index_to_remove = Some(i);
                break;
            }
        }
        
        if let Some(idx) = index_to_remove {
            managers.remove(idx);
            env.storage().persistent().set(&DataKey::Managers, &managers);
            env.events().publish((TOPIC_MANAGER, symbol_short!("remove")), (caller, manager));
        } else {
            panic!("{:?}", Error::ManagerNotFound);
        }
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
    pub fn get_contract_creator(env: Env) -> Address{
        return env.storage().persistent().get::<DataKey, Address>(&DataKey::ScorerFactoryCreator).unwrap_or_else(|| panic!("{:?}", Error::ContractCreatorNotFound));
    }

    /// Removes a scorer contract from the factory's registry
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `caller` - The address that will authenticate the removal of the scorer
    /// * `scorer_address` - The address of the scorer contract to be removed
    /// 
    /// # Panics
    /// * When the caller is not a registered manager (`Error::Unauthorized`)
    /// * When the scorer address is not found in the registry
    pub fn remove_scorer(env: Env, caller: Address, scorer_address: Address) {
        // Require authentication from the caller
        caller.require_auth();

        // Verify caller is a manager
        if !Self::is_manager(env.clone(), caller.clone()) {
            panic!("{:?}", Error::Unauthorized);
        }

        let mut created_scorers = env.storage()
            .persistent()
            .get::<DataKey, Map<Address, (String, String, String)>>(&DataKey::CreatedScorers)
            .unwrap_or_else(|| Map::new(&env));

        // Check if the scorer exists
        if !created_scorers.contains_key(scorer_address.clone()) {
            panic!("{:?}", Error::ScorerNotFound);
        }

        let (scorer_name, scorer_description, icon) = created_scorers.get(scorer_address.clone()).unwrap();
        
        // Remove the scorer from the map
        created_scorers.remove(scorer_address.clone());
        
        // Update storage
        env.storage().persistent().set(&DataKey::CreatedScorers, &created_scorers);
        
        // Emit an event for the removal
        env.events().publish(
            (TOPIC_SCORER, symbol_short!("remove")), 
            (caller, scorer_address, scorer_name, scorer_description, icon)
        );
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    
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
        let (_env, scorer_factory_creator, scorer_factory_client) = setup_contract();
        assert!(scorer_factory_client.is_initialized());
        assert!(scorer_factory_client.is_scorer_factory_creator(&scorer_factory_creator));
    }

    #[test]
    fn test_is_manager() {
        let (_env, scorer_factory_creator, scorer_factory_client) = setup_contract();
        assert!(scorer_factory_client.is_manager(&scorer_factory_creator));
    }

    #[test]
    fn test_get_scorers() {
        let (_env, _scorer_factory_creator, scorer_factory_client) = setup_contract();
        let scorers = scorer_factory_client.get_scorers();
        assert!(scorers.len() == 0);
    }
}