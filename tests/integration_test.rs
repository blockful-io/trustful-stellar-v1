use soroban_sdk::{
    testutils::{Address as _},
    Address, Env, BytesN, Map, String, Vec, Val, Symbol, symbol_short
 };
 use deployer::{Deployer, DeployerClient as DeployerContractClient}; 
 use scorer_factory::{ScorerFactoryContractClient, ScorerFactoryContract};
 use scorer::ScorerContractClient;
 use scorer::{BadgeId, BadgeDetails};
 
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
            
            // Create the badge map with new structure
            let mut scorer_badges = Map::new(&env);
            let badge_id = BadgeId {
                name: String::from_str(&env, "Test Badge"),
                issuer: scorer_factory_creator.clone(),
            };
            
            let badge_details = BadgeDetails {
                score: 100,
                icon: String::from_str(&env, "badge_icon.png"),
            };
            
            scorer_badges.set(badge_id, badge_details);
            let mut init_args: Vec<Val> = Vec::new(&env);
    
            init_args.push_back(scorer_factory_creator.clone().into_val(&env));        
            init_args.push_back(scorer_badges.into_val(&env));
            
            let name = String::from_str(&env, "new_scorer");
            let description = String::from_str(&env, "scorer's description");
            init_args.push_back(name.into_val(&env));
            init_args.push_back(description.into_val(&env));

            // Create the scorer contract
            let scorer_address = scorer_factory_client.create_scorer(
                &scorer_factory_creator,
                &salt,
                &init_fn,
                &init_args,
            );
            
            assert!(!scorer_address.to_string().is_empty());
            
            let expected_event = (
                scorer_factory_client.address.clone(),
                (String::from_str(&env, "scorer"), symbol_short!("create")).into_val(&env),
                (scorer_factory_creator, scorer_address, name, description).into_val(&env)
            );

            assert!(env.events().all().contains(&expected_event), 
                "Expected scorer creation event not found in events list");
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
    fn test_get_scorers() {
        let (env, scorer_factory_creator, scorer_factory_client) = setup_contract();
        let scorers = scorer_factory_client.get_scorers();
        assert!(scorers.len() == 0);

        let salt = BytesN::from_array(&env, &[1; 32]);
        let init_fn = Symbol::new(&env, "initialize");
        
        // Create the badge map with new structure
        let mut scorer_badges = Map::new(&env);
        let badge_id = BadgeId {
            name: String::from_str(&env, "Test Badge"),
            issuer: scorer_factory_creator.clone(),
        };
        
        let badge_details = BadgeDetails {
            score: 100,
            icon: String::from_str(&env, "badge_icon.png"),
        };
        
        scorer_badges.set(badge_id, badge_details);
        let mut init_args: Vec<Val> = Vec::new(&env);

        init_args.push_back(scorer_factory_creator.clone().into_val(&env));        
        init_args.push_back(scorer_badges.into_val(&env));
        init_args.push_back(String::from_str(&env, "new_scorer").into_val(&env));
        init_args.push_back(String::from_str(&env, "scorer's description").into_val(&env));

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
        
        // Create the badge map with new structure
        let mut scorer_badges = Map::new(&env);
        let badge_id = BadgeId {
            name: String::from_str(&env, "Test Badge"),
            issuer: admin.clone(),
        };
        
        let badge_details = BadgeDetails {
            score: 100,
            icon: String::from_str(&env, "badge_icon.png"),
        };
        
        scorer_badges.set(badge_id, badge_details);
        
        let mut scorer_init_args: Vec<Val> = Vec::new(&env);
        scorer_init_args.push_back(admin.clone().into_val(&env));
        scorer_init_args.push_back(scorer_badges.into_val(&env));
        scorer_init_args.push_back(String::from_str(&env, "new_scorer").into_val(&env));
        scorer_init_args.push_back(String::from_str(&env, "scorer's description").into_val(&env));

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

        let name = String::from_str(&env, "new_scorer").into_val(&env);
        let description = String::from_str(&env, "scorer's description").into_val(&env);

        assert_eq!(scorers.len(), 1);
        assert_eq!(scorers.get(scorer_address.clone()).unwrap(), (name, description));

        env.budget().reset_default();

        // Step 9: Create scorer client and verify badges
        let scorer_client = ScorerContractClient::new(&env, &scorer_address);
        let stored_badges = scorer_client.get_badges();
        assert_eq!(stored_badges.len(), 1);
        
        let stored_badge = stored_badges.values().first().unwrap();
        assert_eq!(stored_badge.score, 100);
        assert_eq!(stored_badge.icon, String::from_str(&env, "badge_icon.png"));
        
        // Get the badge key to check the issuer
        let stored_badge_id = stored_badges.keys().first().unwrap();
        assert_eq!(stored_badge_id.name, String::from_str(&env, "Test Badge"));
        assert_eq!(stored_badge_id.issuer, admin);

        // Step 10: Test that new manager can also create a scorer
        let mut new_scorer_badges = Map::new(&env);
        let badge_id = BadgeId {
            name: String::from_str(&env, "Manager Badge"),
            issuer: new_manager.clone(),
        };
        
        let badge_details = BadgeDetails {
            score: 200,
            icon: String::from_str(&env, "badge_icon.png"),
        };
        
        new_scorer_badges.set(badge_id, badge_details);
        
        let mut new_scorer_init_args: Vec<Val> = Vec::new(&env);
        new_scorer_init_args.push_back(new_manager.clone().into_val(&env));
        new_scorer_init_args.push_back(new_scorer_badges.into_val(&env));
        new_scorer_init_args.push_back(String::from_str(&env, "new_scorer").into_val(&env));
        new_scorer_init_args.push_back(String::from_str(&env, "scorer's description").into_val(&env));

        let new_scorer_address = factory_client.create_scorer(
            &new_manager,
            &BytesN::from_array(&env, &[2; 32]),
            &init_fn,
            &new_scorer_init_args,
        );

        // Step 11: Verify second scorer
        let scorers = factory_client.get_scorers();
        assert_eq!(scorers.len(), 2);
        let name: String = String::from_str(&env, "new_scorer").into_val(&env);
        let description: String = String::from_str(&env, "scorer's description").into_val(&env);
        assert_eq!(scorers.get(new_scorer_address.clone()).unwrap(), (name.clone(), description.clone()));

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
        scorer_client.add_user(&user);
        assert!(scorer_client.get_users().contains_key(user.clone()));

        // Step 14: Verify user can add themselves to another scorer
        let new_scorer_client = ScorerContractClient::new(&env, &new_scorer_address);
        new_scorer_client.add_user(&user);
        assert!(new_scorer_client.get_users().contains_key(user.clone()));

        // Step 15: Remove user from first scorer
        scorer_client.remove_user(&user);
        assert_eq!(scorer_client.get_users().get(user.clone()), Some(false));

        // Step 16: Verify user can remove themselves from second scorer
        new_scorer_client.remove_user(&user);
        assert_eq!(new_scorer_client.get_users().get(user.clone()), Some(false));

    }

    #[test]
    fn test_remove_scorer() {
        let (env, admin, factory_client) = setup_contract();
        let manager = Address::generate(&env);
        
        // Add manager
        factory_client.add_manager(&admin, &manager);
        assert!(factory_client.is_manager(&manager));

        // Create a scorer with new badge structure
        let mut scorer_badges = Map::new(&env);
        let badge_id = BadgeId {
            name: String::from_str(&env, "Test Badge"),
            issuer: admin.clone(),
        };
        
        let badge_details = BadgeDetails {
            score: 100,
            icon: String::from_str(&env, "badge_icon.png"),
        };
        
        scorer_badges.set(badge_id, badge_details);
        
        let mut init_args: Vec<Val> = Vec::new(&env);
        init_args.push_back(admin.clone().into_val(&env));
        init_args.push_back(scorer_badges.into_val(&env));
        init_args.push_back(String::from_str(&env, "Test Scorer").into_val(&env));
        init_args.push_back(String::from_str(&env, "A test scorer").into_val(&env));

        let scorer_address = factory_client.create_scorer(
            &admin,
            &BytesN::from_array(&env, &[1_u8; 32]),
            &Symbol::new(&env, "initialize"),
            &init_args
        );

        // Verify scorer was created
        let scorers = factory_client.get_scorers();
        assert!(scorers.contains_key(scorer_address.clone()));

        // Remove the scorer using the manager
        factory_client.remove_scorer(&manager, &scorer_address);

        // Verify scorer was removed
        let scorers_after = factory_client.get_scorers();
        assert!(!scorers_after.contains_key(scorer_address));
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_remove_scorer_unauthorized() {
        let (env, admin, factory_client) = setup_contract();
        let non_manager = Address::generate(&env);
        
        // Create a scorer with new badge structure
        let mut scorer_badges = Map::new(&env);
        let badge_id = BadgeId {
            name: String::from_str(&env, "Test Badge"),
            issuer: admin.clone(),
        };
        
        let badge_details = BadgeDetails {
            score: 100,
            icon: String::from_str(&env, "badge_icon.png"),
        };
        
        scorer_badges.set(badge_id, badge_details);
        
        let mut init_args: Vec<Val> = Vec::new(&env);
        init_args.push_back(admin.clone().into_val(&env));
        init_args.push_back(scorer_badges.into_val(&env));
        init_args.push_back(String::from_str(&env, "Test Scorer").into_val(&env));
        init_args.push_back(String::from_str(&env, "A test scorer").into_val(&env));

        let scorer_address = factory_client.create_scorer(
            &admin,
            &BytesN::from_array(&env, &[1_u8; 32]),
            &Symbol::new(&env, "initialize"),
            &init_args
        );

        // Attempt to remove the scorer with a non-manager (should panic)
        factory_client.remove_scorer(&non_manager, &scorer_address);
    }

    #[test]
    #[should_panic(expected = "ScorerNotFound")]
    fn test_remove_nonexistent_scorer() {
        let (env, admin, factory_client) = setup_contract();

        // Try to remove a non-existent scorer (should panic)
        let nonexistent_scorer = Address::generate(&env);
        factory_client.remove_scorer(&admin, &nonexistent_scorer);
    }
 }