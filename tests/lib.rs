use scrypto::prelude::{Global, Account, Url};
use scrypto_test::prelude::*;
use consultation_blueprint::*;

// =============================================================================
// Test Helpers
// =============================================================================

/// Creates an owner badge and deposits it to a new account
/// Returns (badge_address, owner_account, owner_public_key)
fn create_owner_badge_with_account(
    ledger: &mut LedgerSimulator<NoExtension, InMemorySubstateDatabase>
) -> (ResourceAddress, ComponentAddress, Secp256k1PublicKey) {
    let (public_key, _private_key, owner_account) = ledger.new_allocated_account();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_fungible_resource(
            OwnerRole::None,
            false,
            0,
            FungibleResourceRoles::default(),
            metadata!(),
            Some(dec!(1)),
        )
        .try_deposit_entire_worktop_or_abort(owner_account, None)
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
    let owner_badge = receipt.expect_commit(true).new_resource_addresses()[0];

    (owner_badge, owner_account, public_key)
}

fn create_governance_parameters() -> GovernanceParameters {
    GovernanceParameters {
        temperature_check_days: 7,
        temperature_check_quorum: dec!(1000),
        temperature_check_approval_threshold: dec!("0.5"),
        temperature_check_propose_threshold: dec!(100),
        proposal_length_days: 14,
        proposal_quorum: dec!(5000),
        proposal_approval_threshold: dec!("0.5"),
    }
}

fn create_temp_check_draft() -> TemperatureCheckDraft {
    TemperatureCheckDraft {
        title: "Test Proposal".to_string(),
        description: "A test proposal description".to_string(),
        vote_options: vec![
            ProposalVoteOption {
                id: ProposalVoteOptionId(0),
                label: "For".to_string(),
            },
            ProposalVoteOption {
                id: ProposalVoteOptionId(1),
                label: "Against".to_string(),
            },
        ],
        attachments: vec![],
        rfc_url: Url::of("https://radixtalk.com/proposal/123"),
    }
}

// =============================================================================
// Governance Blueprint Tests
// =============================================================================

#[test]
fn test_governance_instantiate() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (owner_badge, _owner_account, _public_key) = create_owner_badge_with_account(&mut ledger);
    let params = create_governance_parameters();

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            ledger.compile_and_publish(this_package!()),
            "Governance",
            "instantiate",
            manifest_args!(owner_badge, params),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn test_make_temperature_check() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (owner_badge, _owner_account, _public_key) = create_owner_badge_with_account(&mut ledger);
    let params = create_governance_parameters();
    let package_address = ledger.compile_and_publish(this_package!());

    // Instantiate governance
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Governance",
            "instantiate",
            manifest_args!(owner_badge, params),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
    let governance_component = receipt.expect_commit(true).new_component_addresses()[0];

    // Create temperature check
    let draft = create_temp_check_draft();
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            governance_component,
            "make_temperature_check",
            manifest_args!(draft),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();

    // Verify counter increased
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            governance_component,
            "get_temperature_check_count",
            manifest_args!(),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    let count: u64 = receipt.expect_commit_success().output(1);
    assert_eq!(count, 1);
}

#[test]
fn test_vote_on_temperature_check() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (owner_badge, _owner_account, _owner_pk) = create_owner_badge_with_account(&mut ledger);
    let params = create_governance_parameters();
    let package_address = ledger.compile_and_publish(this_package!());

    // Create voter account
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    // Instantiate governance
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Governance",
            "instantiate",
            manifest_args!(owner_badge, params),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    let governance_component = receipt.expect_commit(true).new_component_addresses()[0];

    // Create temperature check
    let draft = create_temp_check_draft();
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            governance_component,
            "make_temperature_check",
            manifest_args!(draft),
        )
        .build();

    ledger.execute_manifest(manifest, vec![]).expect_commit_success();

    // Vote on temperature check
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            governance_component,
            "vote_on_temperature_check",
            manifest_args!(account, 0u64, TemperatureCheckVote::For),
        )
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();
}

#[test]
fn test_cannot_vote_twice_on_temperature_check() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (owner_badge, _owner_account, _owner_pk) = create_owner_badge_with_account(&mut ledger);
    let params = create_governance_parameters();
    let package_address = ledger.compile_and_publish(this_package!());

    // Create voter account
    let (public_key, _private_key, account) = ledger.new_allocated_account();

    // Instantiate governance
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Governance",
            "instantiate",
            manifest_args!(owner_badge, params),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    let governance_component = receipt.expect_commit(true).new_component_addresses()[0];

    // Create temperature check
    let draft = create_temp_check_draft();
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            governance_component,
            "make_temperature_check",
            manifest_args!(draft),
        )
        .build();

    ledger.execute_manifest(manifest, vec![]).expect_commit_success();

    // First vote should succeed
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            governance_component,
            "vote_on_temperature_check",
            manifest_args!(account, 0u64, TemperatureCheckVote::For),
        )
        .build();

    ledger
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&public_key)],
        )
        .expect_commit_success();

    // Second vote should fail
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            governance_component,
            "vote_on_temperature_check",
            manifest_args!(account, 0u64, TemperatureCheckVote::Against),
        )
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_failure();
}

#[test]
fn test_make_proposal_from_temperature_check() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let package_address = ledger.compile_and_publish(this_package!());

    // Create owner account with badge
    let (owner_badge, owner_account, public_key) = create_owner_badge_with_account(&mut ledger);
    let params = create_governance_parameters();

    // Instantiate governance
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "Governance",
            "instantiate",
            manifest_args!(owner_badge, params),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    let governance_component = receipt.expect_commit(true).new_component_addresses()[0];

    // Create temperature check
    let draft = create_temp_check_draft();
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            governance_component,
            "make_temperature_check",
            manifest_args!(draft),
        )
        .build();

    ledger.execute_manifest(manifest, vec![]).expect_commit_success();

    // Elevate to proposal (requires owner badge proof for auth)
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .create_proof_from_account_of_amount(owner_account, owner_badge, dec!(1))
        .call_method(
            governance_component,
            "make_proposal",
            manifest_args!(0u64),
        )
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&public_key)],
    );
    receipt.expect_commit_success();

    // Verify proposal was created
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            governance_component,
            "get_proposal_count",
            manifest_args!(),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    let count: u64 = receipt.expect_commit_success().output(1);
    assert_eq!(count, 1);
}

// =============================================================================
// VoteDelegation Blueprint Tests
// =============================================================================

#[test]
fn test_vote_delegation_instantiate() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (owner_badge, _owner_account, _public_key) = create_owner_badge_with_account(&mut ledger);

    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            ledger.compile_and_publish(this_package!()),
            "VoteDelegation",
            "instantiate",
            manifest_args!(owner_badge),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    receipt.expect_commit_success();
}

#[test]
fn test_make_delegation() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (owner_badge, _owner_account, _owner_pk) = create_owner_badge_with_account(&mut ledger);
    let package_address = ledger.compile_and_publish(this_package!());

    // Create delegator and delegatee accounts
    let (delegator_pk, _delegator_sk, delegator_account) = ledger.new_allocated_account();
    let (_delegatee_pk, _delegatee_sk, delegatee_account) = ledger.new_allocated_account();

    // Instantiate vote delegation
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "VoteDelegation",
            "instantiate",
            manifest_args!(owner_badge),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    let delegation_component = receipt.expect_commit(true).new_component_addresses()[0];

    // Set valid_until to future time
    let valid_until = Instant::new(i64::MAX / 2);

    // Make delegation
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            delegation_component,
            "make_delegation",
            manifest_args!(delegator_account, delegatee_account, dec!("0.5"), valid_until),
        )
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&delegator_pk)],
    );
    receipt.expect_commit_success();

    // Verify delegation exists by checking via get_delegatee_delegators
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            delegation_component,
            "get_delegatee_delegators",
            manifest_args!(delegatee_account, delegator_account),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    let fraction: Option<Decimal> = receipt.expect_commit_success().output(1);
    assert_eq!(fraction, Some(dec!("0.5")));
}

#[test]
fn test_remove_delegation() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (owner_badge, _owner_account, _owner_pk) = create_owner_badge_with_account(&mut ledger);
    let package_address = ledger.compile_and_publish(this_package!());

    // Create delegator and delegatee accounts
    let (delegator_pk, _delegator_sk, delegator_account) = ledger.new_allocated_account();
    let (_delegatee_pk, _delegatee_sk, delegatee_account) = ledger.new_allocated_account();

    // Instantiate vote delegation
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "VoteDelegation",
            "instantiate",
            manifest_args!(owner_badge),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    let delegation_component = receipt.expect_commit(true).new_component_addresses()[0];

    let valid_until = Instant::new(i64::MAX / 2);

    // Make delegation
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            delegation_component,
            "make_delegation",
            manifest_args!(delegator_account, delegatee_account, dec!("0.5"), valid_until),
        )
        .build();

    ledger
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&delegator_pk)],
        )
        .expect_commit_success();

    // Remove delegation
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            delegation_component,
            "remove_delegation",
            manifest_args!(delegator_account, delegatee_account),
        )
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&delegator_pk)],
    );
    receipt.expect_commit_success();

    // Verify delegation was removed
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            delegation_component,
            "get_delegatee_delegators",
            manifest_args!(delegatee_account, delegator_account),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    let fraction: Option<Decimal> = receipt.expect_commit_success().output(1);
    assert_eq!(fraction, None);
}

#[test]
fn test_cannot_delegate_more_than_100_percent() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (owner_badge, _owner_account, _owner_pk) = create_owner_badge_with_account(&mut ledger);
    let package_address = ledger.compile_and_publish(this_package!());

    // Create accounts
    let (delegator_pk, _delegator_sk, delegator_account) = ledger.new_allocated_account();
    let (_delegatee1_pk, _delegatee1_sk, delegatee1_account) = ledger.new_allocated_account();
    let (_delegatee2_pk, _delegatee2_sk, delegatee2_account) = ledger.new_allocated_account();

    // Instantiate vote delegation
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "VoteDelegation",
            "instantiate",
            manifest_args!(owner_badge),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    let delegation_component = receipt.expect_commit(true).new_component_addresses()[0];

    let valid_until = Instant::new(i64::MAX / 2);

    // First delegation of 60%
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            delegation_component,
            "make_delegation",
            manifest_args!(delegator_account, delegatee1_account, dec!("0.6"), valid_until),
        )
        .build();

    ledger
        .execute_manifest(
            manifest,
            vec![NonFungibleGlobalId::from_public_key(&delegator_pk)],
        )
        .expect_commit_success();

    // Second delegation of 50% should fail (60% + 50% > 100%)
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            delegation_component,
            "make_delegation",
            manifest_args!(delegator_account, delegatee2_account, dec!("0.5"), valid_until),
        )
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&delegator_pk)],
    );
    receipt.expect_commit_failure();
}

#[test]
fn test_cannot_delegate_to_self() {
    let mut ledger = LedgerSimulatorBuilder::new().build();
    let (owner_badge, _owner_account, _owner_pk) = create_owner_badge_with_account(&mut ledger);
    let package_address = ledger.compile_and_publish(this_package!());

    // Create account
    let (delegator_pk, _delegator_sk, delegator_account) = ledger.new_allocated_account();

    // Instantiate vote delegation
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_function(
            package_address,
            "VoteDelegation",
            "instantiate",
            manifest_args!(owner_badge),
        )
        .build();

    let receipt = ledger.execute_manifest(manifest, vec![]);
    let delegation_component = receipt.expect_commit(true).new_component_addresses()[0];

    let valid_until = Instant::new(i64::MAX / 2);

    // Try to delegate to self
    let manifest = ManifestBuilder::new()
        .lock_fee_from_faucet()
        .call_method(
            delegation_component,
            "make_delegation",
            manifest_args!(delegator_account, delegator_account, dec!("0.5"), valid_until),
        )
        .build();

    let receipt = ledger.execute_manifest(
        manifest,
        vec![NonFungibleGlobalId::from_public_key(&delegator_pk)],
    );
    receipt.expect_commit_failure();
}
