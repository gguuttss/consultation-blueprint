use scrypto::prelude::*;

pub mod governance;
pub mod vote_delegation;

// =============================================================================
// Shared Types
// =============================================================================

/// Reference to a file stored via radix-file-storage
#[derive(ScryptoSbor, ManifestSbor, Clone, Debug)]
pub struct File {
    pub kvs_address: String,
    pub component_address: ComponentAddress,
    pub file_hash: String,
}

/// Vote option for temperature checks (simple for/against)
#[derive(ScryptoSbor, ManifestSbor, Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemperatureCheckVote {
    For,
    Against,
}

/// Unique identifier for a proposal vote option
#[derive(ScryptoSbor, ManifestSbor, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ProposalVoteOptionId(pub u32);

/// A vote option for proposals (e.g., "For", "Against", "Abstain")
#[derive(ScryptoSbor, ManifestSbor, Clone, Debug)]
pub struct ProposalVoteOption {
    pub id: ProposalVoteOptionId,
    pub label: String,
}

/// Maximum number of attachments per temperature check / proposal
pub const MAX_ATTACHMENTS: usize = 10;
/// Maximum number of vote options per proposal
pub const MAX_VOTE_OPTIONS: usize = 10;
/// Maximum number of selections in a multiple-choice vote
pub const MAX_SELECTIONS: u32 = 5;

// =============================================================================
// Delegation Constants
// =============================================================================

/// Maximum number of delegations a single account can have
pub const MAX_DELEGATIONS: usize = 50;
/// Minimum delegation fraction (1% = 0.01)
pub const MIN_DELEGATION_FRACTION: &str = "0.01";

// =============================================================================
// Governance Types
// =============================================================================

/// Input data for creating a temperature check
#[derive(ScryptoSbor, ManifestSbor, Clone, Debug)]
pub struct TemperatureCheckDraft {
    pub title: String,
    pub description: String,
    pub vote_options: Vec<ProposalVoteOption>,
    pub attachments: Vec<File>,
    pub rfc_url: Url,
    /// Maximum number of options a voter can select in the proposal.
    /// If None, only one option can be selected (single choice).
    /// If Some(n), up to n options can be selected (multiple choice).
    pub max_selections: Option<u32>,
}

/// Governance parameters that control voting behavior
#[derive(ScryptoSbor, ManifestSbor, Clone, Debug)]
pub struct GovernanceParameters {
    pub temperature_check_days: u16,
    pub temperature_check_quorum: Decimal,
    pub temperature_check_approval_threshold: Decimal,
    pub proposal_length_days: u16,
    pub proposal_quorum: Decimal,
    pub proposal_approval_threshold: Decimal,
}

/// Struct used to hold submitted temperature check data
#[derive(ScryptoSbor)]
pub struct TemperatureCheck {
    pub title: String,
    pub description: String,
    pub vote_options: Vec<ProposalVoteOption>,
    pub attachments: Vec<File>,
    pub rfc_url: Url,
    pub quorum: Decimal,
    /// Maximum number of options a voter can select in the proposal.
    /// If None, only one option can be selected (single choice).
    /// If Some(n), up to n options can be selected (multiple choice).
    pub max_selections: Option<u32>,
    pub votes: KeyValueStore<Global<Account>, TemperatureCheckVote>,
    pub approval_threshold: Decimal,
    pub start: Instant,
    pub deadline: Instant,
    pub elevated_proposal_id: Option<u64>,
}

/// Struct for a proposal (GP - Governance Proposal)
#[derive(ScryptoSbor)]
pub struct Proposal {
    pub title: String,
    pub description: String,
    pub vote_options: Vec<ProposalVoteOption>,
    pub attachments: Vec<File>,
    pub rfc_url: Url,
    pub quorum: Decimal,
    /// Maximum number of options a voter can select.
    /// If None, only one option can be selected (single choice).
    /// If Some(n), up to n options can be selected (multiple choice).
    pub max_selections: Option<u32>,
    /// Stores selected option IDs for each voter
    pub votes: KeyValueStore<Global<Account>, Vec<ProposalVoteOptionId>>,
    pub approval_threshold: Decimal,
    pub start: Instant,
    pub deadline: Instant,
    pub temperature_check_id: u64,
}

// =============================================================================
// Delegation Types
// =============================================================================

/// Represents a delegation from one account to another
#[derive(ScryptoSbor, Clone, Debug)]
pub struct Delegation {
    pub delegatee: Global<Account>,
    pub fraction: Decimal,
    pub valid_until: Instant,
}

// =============================================================================
// Events
// =============================================================================

/// Emitted when a temperature check is created
#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct TemperatureCheckCreatedEvent {
    pub temperature_check_id: u64,
    pub title: String,
    pub start: Instant,
    pub deadline: Instant,
}

/// Emitted when a vote is cast on a temperature check
#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct TemperatureCheckVotedEvent {
    pub temperature_check_id: u64,
    pub account: Global<Account>,
    pub vote: TemperatureCheckVote,
}

/// Emitted when a temperature check is elevated to a proposal
#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct ProposalCreatedEvent {
    pub proposal_id: u64,
    pub temperature_check_id: u64,
    pub title: String,
    pub start: Instant,
    pub deadline: Instant,
}

/// Emitted when a vote is cast on a proposal
#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct ProposalVotedEvent {
    pub proposal_id: u64,
    pub account: Global<Account>,
    pub votes: Vec<ProposalVoteOptionId>,
}

/// Emitted when governance parameters are updated
#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct GovernanceParametersUpdatedEvent {
    pub new_params: GovernanceParameters,
}

/// Emitted when a delegation is created or updated
#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct DelegationCreatedEvent {
    pub delegator: Global<Account>,
    pub delegatee: Global<Account>,
    pub fraction: Decimal,
    pub valid_until: Instant,
}

/// Emitted when a delegation is removed
#[derive(ScryptoSbor, ScryptoEvent, Clone, Debug)]
pub struct DelegationRemovedEvent {
    pub delegator: Global<Account>,
    pub delegatee: Global<Account>,
}
