use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, BytesN, Map, String, Vec, Val, Symbol, symbol_short
 };
 use deployer::{Deployer, DeployerClient as DeployerContractClient}; 
 use scorer_factory::{ScorerFactoryContractClient, ScorerFactoryContract};
 use scorer::ScorerBadge;
 use scorer::ScorerContractClient;
 
 soroban_sdk::contractimport!(
    file = "wasm/deployer.wasm"
 );
 
 fn install_scorer_wasm(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "wasm/scorer.wasm"
    );
    e.deployer().upload_contract_wasm(WASM)
 }
 
 fn install_scorer_factory_wasm(e: &Env) -> BytesN<32> {
    soroban_sdk::contractimport!(
        file = "wasm/scorer_factory.wasm"
    );
    e.deployer().upload_contract_wasm(WASM)
 }
 
 mod factory_tests {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Events}, IntoVal};
 

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
        fn test_create_scorer() {
            let (env, scorer_factory_creator, scorer_factory_client) = setup_contract();
            
            let salt = BytesN::from_array(&env, &[1; 32]);
            let init_fn = Symbol::new(&env, "initialize");
            
            // Create the badge map
            let mut scorer_badges = Map::new(&env);
            let badge = ScorerBadge {
                name: String::from_str(&env, "Test Badge"),
                issuer: scorer_factory_creator.clone(),
                score: 100,
            };
            scorer_badges.set(1, badge);
            let mut init_args: Vec<Val> = Vec::new(&env);
    
            init_args.push_back(scorer_factory_creator.clone().into_val(&env));        
            init_args.push_back(scorer_badges.into_val(&env));
            // Create the scorer contract
            let scorer_address = scorer_factory_client.create_scorer(
                &scorer_factory_creator,
                &salt,
                &init_fn,
                &init_args,
            );
            
            assert!(!scorer_address.to_string().is_empty());
            
            // Verify event emission
            assert_eq!(
                env.events().all(),
                soroban_sdk::vec![
                    &env,
                    (
                        scorer_factory_client.address.clone(),
                        (String::from_str(&env, "scorer"), symbol_short!("create")).into_val(&env),
                        (scorer_factory_creator, scorer_address).into_val(&env)
                    ),
                ]
            );
        }
    
    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        
        let admin = Address::generate(&env);
        let deployer_id = env.register_contract(None, Deployer);
        let deployer_client = DeployerContractClient::new(&env, &deployer_id);
 
        let factory_wasm_hash = install_scorer_factory_wasm(&env);
        let scorer_wasm_hash = install_scorer_wasm(&env);
 
        let mut init_args: Vec<Val> = Vec::new(&env);   
        init_args.push_back(admin.clone().into_val(&env));
        init_args.push_back(scorer_wasm_hash.into_val(&env));
 
        let factory_id = deployer_client.deploy(
            &admin,
            &factory_wasm_hash,
            &BytesN::from_array(&env, &[0_u8; 32]),
            &Symbol::new(&env, "initialize"),
            &init_args
        );
 
        let factory_client = ScorerFactoryContractClient::new(&env, &factory_id.0);
        assert!(factory_client.is_initialized());
        assert!(factory_client.is_manager(&admin));
    }
 
   #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_create_scorer_unauthorized() {
        let (env, _scorer_factory_creator, scorer_factory_client) = setup_contract();
        
        // Create an unauthorized address
        let unauthorized_address = Address::generate(&env);
        
        let salt = BytesN::from_array(&env, &[1; 32]);
        let init_fn = Symbol::new(&env, "initialize");
        
        // Create the badge map
        let mut scorer_badges = Map::new(&env);
        let badge = ScorerBadge {
            name: String::from_str(&env, "Test Badge"),
            issuer: unauthorized_address.clone(),
            score: 100,
        };
        scorer_badges.set(1, badge);
        
        let mut init_args: Vec<Val> = Vec::new(&env);
        init_args.push_back(unauthorized_address.clone().into_val(&env));
        init_args.push_back(scorer_badges.into_val(&env));

        // This should panic because unauthorized_address is not a manager
        scorer_factory_client.create_scorer(
            &unauthorized_address,
            &salt,
            &init_fn,
            &init_args,
        );
    }

    #[test]
    fn test_get_scorers() {
        let (env, scorer_factory_creator, scorer_factory_client) = setup_contract();
        let scorers = scorer_factory_client.get_scorers();
        assert!(scorers.len() == 0);

        let salt = BytesN::from_array(&env, &[1; 32]);
        let init_fn = Symbol::new(&env, "initialize");
        
        // Create the badge map
        let mut scorer_badges = Map::new(&env);
        let badge = ScorerBadge {
            name: String::from_str(&env, "Test Badge"),
            issuer: scorer_factory_creator.clone(),
            score: 100,
        };
        scorer_badges.set(1, badge);
        let mut init_args: Vec<Val> = Vec::new(&env);

        init_args.push_back(scorer_factory_creator.clone().into_val(&env));        
        init_args.push_back(scorer_badges.into_val(&env));
        // Create the scorer contract
        let scorer_address = scorer_factory_client.create_scorer(
            &scorer_factory_creator,
            &salt,
            &init_fn,
            &init_args,
        );
        
        assert!(!scorer_address.to_string().is_empty());
        
        let scorers = scorer_factory_client.get_scorers();
        assert!(scorers.len() == 1);    
    }
 }
 
 mod integration_tests {
    use super::*;
    use soroban_sdk::{testutils::{Address as _, Events}, IntoVal};
 
    #[test]
    fn test_integration() {
        let env = Env::default();
        env.mock_all_auths();

        // Generate addresses for testing
        let admin = Address::generate(&env);
        let new_manager = Address::generate(&env);
        let user = Address::generate(&env);
        
        // Step 1: Deploy deployer contract
        let deployer_id: Address = env.register_contract(None, Deployer);
        let deployer_client = DeployerContractClient::new(&env, &deployer_id);

        // Step 2: Call deploy to deploy the factory contract
        let factory_wasm_hash = install_scorer_factory_wasm(&env);
        let scorer_wasm_hash = install_scorer_wasm(&env);
        
        let mut init_args: Vec<Val> = Vec::new(&env);   
        init_args.push_back(admin.clone().into_val(&env));
        init_args.push_back(scorer_wasm_hash.into_val(&env));

        let factory_id = deployer_client.deploy(
            &admin,
            &factory_wasm_hash,
            &BytesN::from_array(&env, &[0_u8; 32]),
            &Symbol::new(&env, "initialize"),
            &init_args
        );

        let factory_client = ScorerFactoryContractClient::new(&env, &factory_id.0);

        // Step 3: Check if factory is initialized
        assert!(factory_client.is_initialized());

        // Step 4: Check if admin is manager
        assert!(factory_client.is_manager(&admin));

        // Step 5: Add a new manager and verify event
        factory_client.add_manager(&admin, &new_manager);
        assert!(factory_client.is_manager(&new_manager));
        assert_eq!(
            env.events().all(),
            soroban_sdk::vec![
                &env,
                (
                    factory_client.address.clone(),
                    (String::from_str(&env, "manager"), symbol_short!("add")).into_val(&env),
                    (admin.clone(), new_manager.clone()).into_val(&env)
                )
            ]
        );

        // Step 6: Create a scorer contract
        let salt = BytesN::from_array(&env, &[1; 32]);
        let init_fn = Symbol::new(&env, "initialize");
        
        // Create the badge map
        let mut scorer_badges = Map::new(&env);
        let badge = ScorerBadge {
            name: String::from_str(&env, "Test Badge"),
            issuer: admin.clone(),
            score: 100,
        };
        scorer_badges.set(1, badge);
        
        let mut scorer_init_args: Vec<Val> = Vec::new(&env);
        scorer_init_args.push_back(admin.clone().into_val(&env));
        scorer_init_args.push_back(scorer_badges.into_val(&env));

        // Create the scorer contract
        let scorer_address = factory_client.create_scorer(
            &admin,
            &salt,
            &init_fn,
            &scorer_init_args,
        );

        // Step 7: Verify scorer was created
        assert!(!scorer_address.to_string().is_empty());

        // Step 8: Get scorers and verify count
        let scorers = factory_client.get_scorers();
        assert_eq!(scorers.len(), 1);
        assert!(scorers.get(scorer_address.clone()).unwrap());

        // Step 9: Create scorer client and verify badges
        let scorer_client = ScorerContractClient::new(&env, &scorer_address);
        let stored_badges = scorer_client.get_badges();
        assert_eq!(stored_badges.len(), 1);
        
        let stored_badge = stored_badges.values().first().unwrap();
        assert_eq!(stored_badge.name, String::from_str(&env, "Test Badge"));
        assert_eq!(stored_badge.issuer, admin);
        assert_eq!(stored_badge.score, 100);

        // Step 10: Test that new manager can also create a scorer
        let mut new_scorer_badges = Map::new(&env);
        let new_badge = ScorerBadge {
            name: String::from_str(&env, "Manager Badge"),
            issuer: new_manager.clone(),
            score: 200,
        };
        new_scorer_badges.set(1, new_badge);
        
        let mut new_scorer_init_args: Vec<Val> = Vec::new(&env);
        new_scorer_init_args.push_back(new_manager.clone().into_val(&env));
        new_scorer_init_args.push_back(new_scorer_badges.into_val(&env));

        let new_scorer_address = factory_client.create_scorer(
            &new_manager,
            &BytesN::from_array(&env, &[2; 32]),
            &init_fn,
            &new_scorer_init_args,
        );

        // Step 11: Verify second scorer
        let scorers = factory_client.get_scorers();
        assert_eq!(scorers.len(), 2);
        assert!(scorers.get(new_scorer_address.clone()).unwrap());

        // Step 12: Remove manager and verify event
        factory_client.remove_manager(&admin, &new_manager);
        assert!(!factory_client.is_manager(&new_manager));
        assert!(env.events().all().contains(&(
            factory_client.address.clone(),
            (String::from_str(&env, "manager"), symbol_short!("remove")).into_val(&env),
            (admin.clone(), new_manager.clone()).into_val(&env)
        )));

        // Step 13: Add user to scorer
        let user = Address::generate(&env);
        scorer_client.add_user(&admin, &user);
        assert!(scorer_client.get_users().contains_key(user.clone()));

        // Step 14: Verify user can add themselves to another scorer
        let new_scorer_client = ScorerContractClient::new(&env, &new_scorer_address);
        new_scorer_client.add_user(&user, &user);
        assert!(new_scorer_client.get_users().contains_key(user.clone()));

        // Step 15: Remove user from first scorer
        scorer_client.remove_user(&admin, &user);
        assert_eq!(scorer_client.get_users().get(user.clone()), Some(false));

        // Step 16: Verify user can remove themselves from second scorer
        new_scorer_client.remove_user(&user, &user);
        assert_eq!(new_scorer_client.get_users().get(user.clone()), Some(false));


    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_removed_manager_cannot_create_scorer() {
        let env = Env::default();
        env.mock_all_auths();

        // Setup initial contracts and addresses
        let admin = Address::generate(&env);
        let manager_to_remove = Address::generate(&env);
        
        // Deploy factory contract
        let deployer_id = env.register_contract(None, Deployer);
        let deployer_client = DeployerContractClient::new(&env, &deployer_id);
        
        let factory_wasm_hash = install_scorer_factory_wasm(&env);
        let scorer_wasm_hash = install_scorer_wasm(&env);
        
        let mut init_args: Vec<Val> = Vec::new(&env);   
        init_args.push_back(admin.clone().into_val(&env));
        init_args.push_back(scorer_wasm_hash.into_val(&env));

        let factory_id = deployer_client.deploy(
            &admin,
            &factory_wasm_hash,
            &BytesN::from_array(&env, &[0_u8; 32]),
            &Symbol::new(&env, "initialize"),
            &init_args
        );

        let factory_client = ScorerFactoryContractClient::new(&env, &factory_id.0);

        // Add manager
        factory_client.add_manager(&admin, &manager_to_remove);
        assert!(factory_client.is_manager(&manager_to_remove));

        // Remove manager
        factory_client.remove_manager(&admin, &manager_to_remove);
        assert!(!factory_client.is_manager(&manager_to_remove));

        // Attempt to create scorer with removed manager (should panic)
        let mut scorer_badges = Map::new(&env);
        let badge = ScorerBadge {
            name: String::from_str(&env, "Unauthorized Badge"),
            issuer: manager_to_remove.clone(),
            score: 100,
        };
        scorer_badges.set(1, badge);
        
        let mut init_args: Vec<Val> = Vec::new(&env);
        init_args.push_back(manager_to_remove.clone().into_val(&env));
        init_args.push_back(scorer_badges.into_val(&env));

        // This call should panic with "Unauthorized"
        factory_client.create_scorer(
            &manager_to_remove,
            &BytesN::from_array(&env, &[1; 32]),
            &Symbol::new(&env, "initialize"),
            &init_args,
        );
    }
 }