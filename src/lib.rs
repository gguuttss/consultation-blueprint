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

/// Input data for creating a temperature check
#[derive(ScryptoSbor, ManifestSbor, Clone, Debug)]
pub struct TemperatureCheckDraft {
    pub title: String,
    pub description: String,
    pub vote_options: Vec<ProposalVoteOption>,
    pub attachments: Vec<File>,
    pub rfc_url: Url,
}

/// Governance parameters that control voting behavior
#[derive(ScryptoSbor, ManifestSbor, Clone, Debug)]
pub struct GovernanceParameters {
    pub temperature_check_days: u16,
    pub temperature_check_quorum: Decimal,
    pub temperature_check_approval_threshold: Decimal,
    pub temperature_check_propose_threshold: Decimal,
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
    pub votes: KeyValueStore<Global<Account>, TemperatureCheckVote>,
    pub approval_threshold: Decimal,
    pub start: Instant,
    pub deadline: Instant,
    pub elevated_proposal_id: Option<u64>,
}

/// Struct for a proposal (RFP)
#[derive(ScryptoSbor)]
pub struct Proposal {
    pub title: String,
    pub description: String,
    pub vote_options: Vec<ProposalVoteOption>,
    pub attachments: Vec<File>,
    pub rfc_url: Url,
    pub quorum: Decimal,
    pub votes: KeyValueStore<Global<Account>, ProposalVoteOptionId>,
    pub approval_threshold: Decimal,
    pub start: Instant,
    pub deadline: Instant,
    pub temperature_check_id: u64,
}

/// Represents a delegation from one account to another
#[derive(ScryptoSbor, Clone, Debug)]
pub struct Delegation {
    pub delegatee: Global<Account>,
    pub fraction: Decimal,
    pub valid_until: Instant,
}
