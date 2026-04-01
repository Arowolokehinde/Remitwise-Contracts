#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as AddressTrait,
    Address, Env, String, Vec,
};

fn setup() -> (Env, InsuranceClient<'static>, Address) {
    let env = Env::default();
    let contract_id = env.register_contract(None, Insurance);
    let client = InsuranceClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    client.initialize(&owner);
    env.mock_all_auths();
    (env, client, owner)
}

fn short_name(env: &Env) -> String {
    String::from_str(env, "ShortName")
}

#[test]
fn test_create_policy_succeeds() {
    let (env, client, owner) = setup();
    let name = String::from_str(&env, "Health Policy");
    let coverage_type = CoverageType::Health;
    let policy_id = client.create_policy(
        &owner,
        &name,
        &coverage_type,
        &1_000_000i128,
        &10_000_000i128,
        &None,
    );
    assert_eq!(policy_id, 1);
}

#[test]
fn test_pay_premium_success() {
    let (env, client, owner) = setup();
    let policy_id = client.create_policy(
        &owner,
        &short_name(&env),
        &CoverageType::Health,
        &1_000_000i128,
        &10_000_000i128,
        &None,
    );
    client.pay_premium(&owner, &policy_id);
}

#[test]
fn test_deactivate_policy_success() {
    let (env, client, owner) = setup();
    let policy_id = client.create_policy(
        &owner,
        &short_name(&env),
        &CoverageType::Health,
        &1_000_000i128,
        &10_000_000i128,
        &None,
    );
    client.deactivate_policy(&owner, &policy_id);
    let policy = client.get_policy(&policy_id).unwrap();
    assert!(!policy.active);
}

#[test]
fn test_get_active_policies() {
    let (env, client, owner) = setup();
    client.create_policy(&owner, &short_name(&env), &CoverageType::Health, &1_000_000i128, &10_000_000i128, &None);
    client.create_policy(&owner, &short_name(&env), &CoverageType::Life, &1_000_000i128, &50_000_000i128, &None);
    let active = client.get_active_policies(&owner, &0, &10);
    assert_eq!(active.count, 2);
}

#[test]
fn test_get_total_monthly_premium() {
    let (env, client, owner) = setup();
    client.create_policy(&owner, &short_name(&env), &CoverageType::Health, &1_000_000i128, &10_000_000i128, &None);
    client.create_policy(&owner, &short_name(&env), &CoverageType::Life, &2_000_000i128, &50_000_000i128, &None);
    assert_eq!(client.get_total_monthly_premium(&owner), 3_000_000i128);
}

#[test]
fn test_health_premium_at_minimum_boundary() {
    let (env, client, owner) = setup();
    client.create_policy(
        &owner,
        &short_name(&env),
        &CoverageType::Health,
        &100i128, // Matches lib.rs check: if monthly_premium < 100 { return Err(...); }
        &10_000_000i128,
        &None,
    );
}

#[test]
fn test_health_premium_below_minimum_fails() {
    let (env, client, owner) = setup();
    let result = client.try_create_policy(
        &owner,
        &short_name(&env),
        &CoverageType::Health,
        &99i128,
        &10_000_000i128,
        &None,
    );
    assert_eq!(result, Err(Ok(InsuranceError::InvalidAmount)));
}

#[test]
fn test_life_premium_at_minimum_boundary() {
    let (env, client, owner) = setup();
    client.create_policy(
        &owner,
        &short_name(&env),
        &CoverageType::Life,
        &500i128,
        &10_000i128,
        &None,
    );
}

#[test]
fn test_life_premium_below_minimum_fails() {
    let (env, client, owner) = setup();
    let result = client.try_create_policy(
        &owner,
        &short_name(&env),
        &CoverageType::Life,
        &499i128,
        &10_000i128,
        &None,
    );
    assert_eq!(result, Err(Ok(InsuranceError::InvalidAmount)));
}

#[test]
fn test_life_coverage_below_minimum_fails() {
    let (env, client, owner) = setup();
    let result = client.try_create_policy(
        &owner,
        &short_name(&env),
        &CoverageType::Life,
        &500i128,
        &9_999i128,
        &None,
    );
    assert_eq!(result, Err(Ok(InsuranceError::InvalidAmount)));
}

#[test]
fn test_property_premium_at_minimum_boundary() {
    let (env, client, owner) = setup();
    client.create_policy(
        &owner,
        &short_name(&env),
        &CoverageType::Property,
        &200i128,
        &10_000_000i128,
        &None,
    );
}

#[test]
fn test_property_premium_below_minimum_fails() {
    let (env, client, owner) = setup();
    let result = client.try_create_policy(
        &owner,
        &short_name(&env),
        &CoverageType::Property,
        &199i128,
        &10_000_000i128,
        &None,
    );
    assert_eq!(result, Err(Ok(InsuranceError::InvalidAmount)));
}

#[test]
fn test_create_policy_empty_name_fails() {
    let (env, client, owner) = setup();
    let result = client.try_create_policy(
        &owner,
        &String::from_str(&env, ""),
        &CoverageType::Health,
        &1_000_000i128,
        &10_000_000i128,
        &None,
    );
    assert_eq!(result, Err(Ok(InsuranceError::InvalidName)));
}

#[test]
fn test_create_policy_long_name_fails() {
    let (env, client, owner) = setup();
    // Manual creation of a 65-character string if possible, or just use a known long one.
    // Actually, create_policy checks name.len() > 64.
    let long_name = String::from_str(&env, "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA");
    let result = client.try_create_policy(
        &owner,
        &long_name,
        &CoverageType::Health,
        &1_000_000i128,
        &10_000_000i128,
        &None,
    );
    assert_eq!(result, Err(Ok(InsuranceError::InvalidName)));
}

#[test]
fn test_batch_pay_premiums() {
    let (env, client, owner) = setup();
    let p1 = client.create_policy(&owner, &short_name(&env), &CoverageType::Health, &1_000_000i128, &10_000_000i128, &None);
    let p2 = client.create_policy(&owner, &short_name(&env), &CoverageType::Life, &1_000_000i128, &50_000_000i128, &None);
    let mut ids = Vec::new(&env);
    ids.push_back(p1);
    ids.push_back(p2);
    let count = client.batch_pay_premiums(&owner, &ids);
    assert_eq!(count, 2);
}

#[test]
fn test_add_tags_to_policy() {
    let (env, client, owner) = setup();
    let policy_id = client.create_policy(&owner, &short_name(&env), &CoverageType::Health, &1_000_000i128, &10_000_000i128, &None);
    let mut tags = Vec::new(&env);
    tags.push_back(String::from_str(&env, "tag1"));
    tags.push_back(String::from_str(&env, "tag2"));
    client.add_tags_to_policy(&owner, &policy_id, &tags);
    let policy = client.get_policy(&policy_id).unwrap();
    assert_eq!(policy.tags.len(), 2);
}

#[test]
fn test_remove_tags_from_policy() {
    let (env, client, owner) = setup();
    let policy_id = client.create_policy(&owner, &short_name(&env), &CoverageType::Health, &1_000_000i128, &10_000_000i128, &None);
    let mut tags = Vec::new(&env);
    let t1 = String::from_str(&env, "tag1");
    let t2 = String::from_str(&env, "tag2");
    tags.push_back(t1.clone());
    tags.push_back(t2.clone());
    client.add_tags_to_policy(&owner, &policy_id, &tags);
    
    let mut to_remove = Vec::new(&env);
    to_remove.push_back(t1);
    client.remove_tags_from_policy(&owner, &policy_id, &to_remove);
    
    let policy = client.get_policy(&policy_id).unwrap();
    assert_eq!(policy.tags.len(), 1);
    assert_eq!(policy.tags.get(0).unwrap(), t2);
}

#[test]
#[should_panic(expected = "not initialized")]
fn test_uninitialized_panic() {
    let env = Env::default();
    let contract_id = env.register_contract(None, Insurance);
    let client = InsuranceClient::new(&env, &contract_id);
    let owner = Address::generate(&env);
    client.create_policy(&owner, &String::from_str(&env, "Name"), &CoverageType::Health, &1_000_000, &10_000_000, &None);
}
