use soroban_sdk::{
    contract, contractimpl, Address, BytesN, Env, Symbol, Val, Vec,
};

#[contract]
pub struct Deployer;

#[contractimpl]
impl Deployer {
    /// Deploy the contract Wasm and after deployment invoke the init function
    /// of the contract with the given arguments.
    ///
    /// This has to be authorized by `deployer` (unless the `Deployer` instance
    /// itself is used as deployer). This way the whole operation is atomic
    /// and it's not possible to frontrun the contract initialization.
    ///
    /// Returns the contract address and result of the init function.
    pub fn deploy(
        env: Env,
        deployer: Address,
        wasm_hash: BytesN<32>,
        salt: BytesN<32>,
        init_fn: Symbol,
        init_args: Vec<Val>,
    ) -> (Address, Val) {
        // Skip authorization if deployer is the current contract.
        if deployer != env.current_contract_address() {
            deployer.require_auth();
        }

        // Deploy the contract using the uploaded Wasm with given hash.
        let deployed_address = env
            .deployer()
            .with_address(deployer, salt)
            .deploy(wasm_hash);

        // Invoke the init function with the given arguments.
        let res: Val = env.invoke_contract(&deployed_address, &init_fn, init_args);
        
        // Return the contract ID of the deployed contract and the result of
        // invoking the init result.
        (deployed_address, res)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, BytesN as _}, vec, IntoVal};
    mod scorer_contract {
        soroban_sdk::contractimport!(
            file = "wasm/trustful_stellar_v1.wasm"
        );
    }
    #[test]
    fn test_deploy_scorer() {
        let env = Env::default();
        env.mock_all_auths();

        // Test variables
        let scorer_creator = Address::generate(&env);
        // Deploy the generic deployer contract
        let deployer_address = env.register_contract(None, Deployer);
        let deployer = DeployerClient::new(&env, &deployer_address);

        // Prepare initialization arguments
        let init_args = vec![&env, scorer_creator.into_val(&env)];

        let init_fn = Symbol::new(&env, "initialize");
        
        // Get the WASM hash of the Scorer contract
        let wasm_hash = env.deployer().upload_contract_wasm(scorer_contract::WASM);
        let salt = BytesN::random(&env);

        // Deploy and initialize the scorer contract atomically
        let (_scorer_address, _) = deployer.deploy(
            &scorer_creator,
            &wasm_hash,
            &salt,
            &init_fn,
            &init_args,
        );
    }
}
