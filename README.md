# Consultation / Governance Scrypto Blueprints

A governance system for Radix DLT that enables community-driven decision making through a structured proposal process.

## Overview

Governance happens in a 3-part procedure:

1. **Request for Comment (RFC)**: A draft proposal posted off-chain (e.g., [RadixTalk](https://radixtalk.com))
2. **Temperature Check**: Pushing proposal details on-chain and voting on whether it merits a full vote
3. **Request for Proposal (RFP)**: A passed temperature check becomes a formal proposal for community voting

Vote counting happens off-chain. Voting power is determined by LSU holdings converted to XRD at the **start of the vote**. Users can delegate their voting power to others.

## Architecture

The system is split into two components for modularity:

| Component | Purpose |
|-----------|---------|
| **Governance** | Manages temperature checks, proposals, and voting |
| **VoteDelegation** | Manages vote delegation between accounts |

This separation allows upgrading the Governance component without requiring users to re-establish their delegations.

## Building

```bash
scrypto build
```

## Testing

```bash
scrypto test
```

## Governance Component

### Instantiation

```rust
Governance::instantiate(
    owner_badge: ResourceAddress,
    governance_parameters: GovernanceParameters,
) -> Global<Governance>
```

### Parameters

```rust
GovernanceParameters {
    temperature_check_days: u16,              // Duration of temp check voting
    temperature_check_quorum: Decimal,        // Min XRD for valid result
    temperature_check_approval_threshold: Decimal, // Fraction needed to pass
    temperature_check_propose_threshold: Decimal,  // XRD needed to create temp check
    proposal_length_days: u16,                // Duration of proposal voting
    proposal_quorum: Decimal,                 // Min XRD for valid result
    proposal_approval_threshold: Decimal,     // Fraction needed to pass
}
```

### Methods

| Method | Access | Description |
|--------|--------|-------------|
| `make_temperature_check(draft)` | PUBLIC | Create a temperature check from an RFC |
| `vote_on_temperature_check(account, id, vote)` | PUBLIC | Vote For/Against on a temp check |
| `make_proposal(temperature_check_id)` | OWNER | Elevate a temp check to a proposal |
| `vote_on_proposal(account, id, vote)` | PUBLIC | Vote on a proposal |
| `update_governance_parameters(params)` | OWNER | Update governance parameters |
| `get_temperature_check_count()` | PUBLIC | Get total temperature checks |
| `get_proposal_count()` | PUBLIC | Get total proposals |
| `get_governance_parameters()` | PUBLIC | Get current parameters |

### Creating a Temperature Check

```rust
TemperatureCheckDraft {
    title: String,
    description: String,
    vote_options: Vec<ProposalVoteOption>,  // Options for the eventual proposal
    attachments: Vec<File>,                  // On-chain file references
    rfc_url: Url,                           // Link to off-chain RFC
}
```

## VoteDelegation Component

### Instantiation

```rust
VoteDelegation::instantiate(
    owner_badge: ResourceAddress,
) -> Global<VoteDelegation>
```

### Methods

| Method | Access | Description |
|--------|--------|-------------|
| `make_delegation(delegator, delegatee, fraction, valid_until)` | PUBLIC | Delegate voting power |
| `remove_delegation(delegator, delegatee)` | PUBLIC | Remove a delegation |
| `get_delegations(delegator)` | PUBLIC | Get all delegations for an account |
| `get_delegatee_delegators(delegatee, delegator)` | PUBLIC | Get delegation fraction |

### Delegation Rules

- Fraction must be between 0 (exclusive) and 1 (inclusive)
- Total delegation cannot exceed 100%
- Cannot delegate to yourself
- Delegation must have a future expiry

## Off-Chain Vote Counting

To count votes for a temperature check or proposal:

1. Query the `votes` KVS to get all accounts that voted and their votes
2. For each voter, query VoteDelegation's `delegatees` KVS to find accounts they can vote for
3. Query VoteDelegation's `delegators` KVS to adjust voting power for delegated fractions
4. Query LSU holdings of all participating accounts at the vote start time
5. Calculate final vote tallies

The `delegatees` and `delegators` KVSs can grow large, but we only need to query entries for accounts that actually voted.

## Data Structures

### File Reference

For on-chain file storage via [radix-file-storage](https://github.com/thereturnofyo/radix-file-storage):

```rust
File {
    kvs_address: String,
    component_address: ComponentAddress,
    file_hash: String,
}
```

### Vote Types

```rust
// For temperature checks
enum TemperatureCheckVote {
    For,
    Against,
}

// For proposals
struct ProposalVoteOptionId(u32);

struct ProposalVoteOption {
    id: ProposalVoteOptionId,
    label: String,  // e.g., "For", "Against", "Abstain"
}
```

### Delegation

```rust
Delegation {
    delegatee: Global<Account>,
    fraction: Decimal,
    valid_until: Instant,
}
```

## License

MIT
