#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    // Helper to create test environment
    fn setup_test() -> (Env, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let owner = Address::generate(&env);
        (env, owner)
    }

    // Helper to initialize split and create audit entries
    fn create_audit_log(env: &Env, owner: &Address, num_entries: u32) {
        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(env, &contract_id);
        let usdc = Address::generate(env);

        // Initialize
        let _init = client.initialize_split(
            owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        // Create audit entries by performing operations
        for i in 0..num_entries {
            let nonce = i as u64;
            let amount = 100_0000000i128 + i as i128;

            let deadline = env.ledger().timestamp() + 3600;
            let request_hash = RemittanceSplit::compute_request_hash(
                symbol_short!("distrib"),
                owner.clone(),
                nonce,
                amount,
                deadline,
            );

            let accounts = AccountGroup {
                spending: Address::generate(env),
                savings: Address::generate(env),
                bills: Address::generate(env),
                insurance: Address::generate(env),
            };

            // Each operation creates an audit entry
            let _result = client.try_distribute_usdc(
                &usdc,
                owner,
                nonce,
                deadline,
                request_hash,
                &accounts,
                amount,
            );
        }
    }

    #[test]
    fn test_get_audit_log_empty_page() {
        let (env, owner) = setup_test();

        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(&env, &contract_id);
        let usdc = Address::generate(&env);

        let _init = client.initialize_split(
            &owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        // Query empty audit log
        let page = client.get_audit_log(&0, &20);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.next_cursor, 0);
        assert_eq!(page.count, 0);
    }

    #[test]
    fn test_get_audit_log_single_entry() {
        let (env, owner) = setup_test();

        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(&env, &contract_id);
        let usdc = Address::generate(&env);

        let _init = client.initialize_split(
            &owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        create_audit_log(&env, &owner, 1);

        // Get first page
        let page = client.get_audit_log(&0, &20);
        assert_eq!(page.items.len(), 1);
        assert_eq!(page.next_cursor, 0); // No more pages
        assert_eq!(page.count, 1);
    }

    #[test]
    fn test_get_audit_log_from_index_beyond_length() {
        let (env, owner) = setup_test();

        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(&env, &contract_id);
        let usdc = Address::generate(&env);

        let _init = client.initialize_split(
            &owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        create_audit_log(&env, &owner, 5);

        // Query starting at index 10 (beyond the 5 entries)
        let page = client.get_audit_log(&10, &20);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.next_cursor, 0);
        assert_eq!(page.count, 0);
    }

    #[test]
    fn test_get_audit_log_u32_max_cursor() {
        let (env, owner) = setup_test();

        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(&env, &contract_id);
        let usdc = Address::generate(&env);

        let _init = client.initialize_split(
            &owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        // Create a few audit entries
        create_audit_log(&env, &owner, 5);

        // Query with u32::MAX cursor - should safely return empty page
        let page = client.get_audit_log(&u32::MAX, &20);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.next_cursor, 0);
        assert_eq!(page.count, 0);
    }

    #[test]
    fn test_get_audit_log_u32_max_limit() {
        let (env, owner) = setup_test();

        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(&env, &contract_id);
        let usdc = Address::generate(&env);

        let _init = client.initialize_split(
            &owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        // Create max possible audit entries (100)
        create_audit_log(&env, &owner, 100);

        // Query with u32::MAX limit - should be clamped to MAX_PAGE_LIMIT (50)
        let page = client.get_audit_log(&0, &u32::MAX);
        assert_eq!(page.items.len(), 50);
        assert_eq!(page.next_cursor, 50); // More pages exist
        assert_eq!(page.count, 50);
    }

    #[test]
    fn test_get_audit_log_saturating_add_overflow() {
        let (env, owner) = setup_test();

        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(&env, &contract_id);
        let usdc = Address::generate(&env);

        let _init = client.initialize_split(
            &owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        // Create 100 audit entries
        create_audit_log(&env, &owner, 100);

        // Query: from_index = u32::MAX - 10, limit = 50
        // saturating_add will overflow but be clamped by min(len)
        let page = client.get_audit_log(&(u32::MAX - 10), &50);
        assert_eq!(page.items.len(), 0); // Beyond log length
        assert_eq!(page.next_cursor, 0);
        assert_eq!(page.count, 0);
    }

    #[test]
    fn test_get_audit_log_pagination_determinism() {
        let (env, owner) = setup_test();

        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(&env, &contract_id);
        let usdc = Address::generate(&env);

        let _init = client.initialize_split(
            &owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        // Create 50 audit entries
        create_audit_log(&env, &owner, 50);

        // Query same parameters twice - should get identical results
        let page1 = client.get_audit_log(&10, &20);
        let page2 = client.get_audit_log(&10, &20);

        assert_eq!(page1.items.len(), page2.items.len());
        assert_eq!(page1.next_cursor, page2.next_cursor);
        assert_eq!(page1.count, page2.count);

        // Verify items are identical
        for i in 0..page1.items.len() {
            let item1 = page1.items.get(i as u32).unwrap();
            let item2 = page2.items.get(i as u32).unwrap();
            assert_eq!(item1.operation, item2.operation);
            assert_eq!(item1.caller, item2.caller);
            assert_eq!(item1.timestamp, item2.timestamp);
            assert_eq!(item1.success, item2.success);
        }
    }

    #[test]
    fn test_get_audit_log_exact_boundary() {
        let (env, owner) = setup_test();

        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(&env, &contract_id);
        let usdc = Address::generate(&env);

        let _init = client.initialize_split(
            &owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        // Create exactly 100 audit entries
        create_audit_log(&env, &owner, 100);

        // Query from_index=0, limit=100 (clamped to 50)
        let page = client.get_audit_log(&0, &100);
        assert_eq!(page.items.len(), 50);
        assert_eq!(page.next_cursor, 50);
        assert_eq!(page.count, 50);

        // Query second page
        let page2 = client.get_audit_log(&50, &100);
        assert_eq!(page2.items.len(), 50);
        assert_eq!(page2.next_cursor, 100);
        assert_eq!(page2.count, 50);

        // Query third page (should be empty)
        let page3 = client.get_audit_log(&100, &50);
        assert_eq!(page3.items.len(), 0);
        assert_eq!(page3.next_cursor, 0);
        assert_eq!(page3.count, 0);
    }

    #[test]
    fn test_get_audit_log_middle_page() {
        let (env, owner) = setup_test();

        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(&env, &contract_id);
        let usdc = Address::generate(&env);

        let _init = client.initialize_split(
            &owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        // Create 75 audit entries
        create_audit_log(&env, &owner, 75);

        // Query middle page: from_index=25, limit=20
        let page = client.get_audit_log(&25, &20);
        assert_eq!(page.items.len(), 20);
        assert_eq!(page.next_cursor, 45);
        assert_eq!(page.count, 20);

        // Query next page: from_index=45, limit=20
        let page2 = client.get_audit_log(&45, &20);
        assert_eq!(page2.items.len(), 20);
        assert_eq!(page2.next_cursor, 65);
        assert_eq!(page2.count, 20);

        // Query final page: from_index=65, limit=20
        let page3 = client.get_audit_log(&65, &20);
        assert_eq!(page3.items.len(), 10); // Only 10 items left
        assert_eq!(page3.next_cursor, 0); // No more pages
        assert_eq!(page3.count, 10);
    }

    #[test]
    fn test_get_audit_log_zero_limit_defaults_to_default() {
        let (env, owner) = setup_test();

        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(&env, &contract_id);
        let usdc = Address::generate(&env);

        let _init = client.initialize_split(
            &owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        // Create 50 audit entries
        create_audit_log(&env, &owner, 50);

        // Query with limit=0 (should use default of 20)
        let page = client.get_audit_log(&0, &0);
        assert_eq!(page.items.len(), 20);
        assert_eq!(page.next_cursor, 20);
        assert_eq!(page.count, 20);
    }

    #[test]
    fn test_get_audit_log_no_panic_on_extreme_values() {
        let (env, owner) = setup_test();

        let contract_id = env.register_contract(None, RemittanceSplit);
        let client = RemittanceSplitClient::new(&env, &contract_id);
        let usdc = Address::generate(&env);

        let _init = client.initialize_split(
            &owner,
            0,
            &usdc,
            25,
            25,
            25,
            25,
        );

        // Create 100 audit entries
        create_audit_log(&env, &owner, 100);

        // These queries should not panic despite extreme values
        let combinations = vec![
            (0, u32::MAX),
            (u32::MAX, 1),
            (u32::MAX, u32::MAX),
            (u32::MAX - 50, 100),
            (50, u32::MAX - 50),
        ];

        for (from_index, limit) in combinations {
            let page = client.get_audit_log(&from_index, &limit);
            // Should return a valid page, not panic
            assert!(page.items.len() <= 50); // Never exceeds MAX_PAGE_LIMIT
        }
    }
}
