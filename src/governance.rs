use scrypto::prelude::*;
use crate::{
    File, GovernanceParameters, Proposal, ProposalVoteOption, ProposalVoteOptionId,
    TemperatureCheck, TemperatureCheckDraft, TemperatureCheckVote, MAX_ATTACHMENTS, MAX_VOTE_OPTIONS,
};

#[blueprint]
mod governance {
    use super::*;

    enable_method_auth! {
        roles {
            owner => updatable_by: [];
        },
        methods {
            // Public methods
            make_temperature_check => PUBLIC;
            vote_on_temperature_check => PUBLIC;
            vote_on_proposal => PUBLIC;
            get_governance_parameters => PUBLIC;
            get_temperature_check_count => PUBLIC;
            get_proposal_count => PUBLIC;
            // Owner-only methods
            make_proposal => restrict_to: [owner];
            update_governance_parameters => restrict_to: [owner];
        }
    }

    struct Governance {
        pub governance_parameters: GovernanceParameters,
        pub temperature_checks: KeyValueStore<u64, TemperatureCheck>,
        pub temperature_check_count: u64,
        pub proposals: KeyValueStore<u64, Proposal>,
        pub proposal_count: u64,
    }

    impl Governance {
        /// Instantiates the governance component with the given owner badge
        pub fn instantiate(
            owner_badge: ResourceAddress,
            governance_parameters: GovernanceParameters,
        ) -> Global<Governance> {
            Self {
                governance_parameters,
                temperature_checks: KeyValueStore::new(),
                temperature_check_count: 0,
                proposals: KeyValueStore::new(),
                proposal_count: 0,
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(owner_badge))))
            .roles(roles! {
                owner => rule!(require(owner_badge));
            })
            .globalize()
        }

        /// Creates a temperature check from the draft
        /// Returns the ID of the created temperature check
        pub fn make_temperature_check(&mut self, draft: TemperatureCheckDraft) -> u64 {
            // Validate inputs
            assert!(
                !draft.title.is_empty(),
                "Temperature check title cannot be empty"
            );
            assert!(
                !draft.description.is_empty(),
                "Temperature check description cannot be empty"
            );
            assert!(
                !draft.vote_options.is_empty(),
                "Temperature check must have at least one vote option"
            );
            assert!(
                draft.vote_options.len() <= MAX_VOTE_OPTIONS,
                "Too many vote options (max {})",
                MAX_VOTE_OPTIONS
            );
            assert!(
                draft.attachments.len() <= MAX_ATTACHMENTS,
                "Too many attachments (max {})",
                MAX_ATTACHMENTS
            );

            let id = self.temperature_check_count;
            self.temperature_check_count += 1;

            let now = Clock::current_time_rounded_to_seconds();
            let deadline = now.add_days(self.governance_parameters.temperature_check_days as i64).unwrap();

            let temperature_check = TemperatureCheck {
                title: draft.title,
                description: draft.description,
                vote_options: draft.vote_options,
                attachments: draft.attachments,
                rfc_url: draft.rfc_url,
                quorum: self.governance_parameters.temperature_check_quorum,
                votes: KeyValueStore::new(),
                approval_threshold: self.governance_parameters.temperature_check_approval_threshold,
                start: now,
                deadline,
                elevated_proposal_id: None,
            };

            self.temperature_checks.insert(id, temperature_check);

            id
        }

        /// Elevates a temperature check to a proposal (RFP)
        /// Only callable by the owner
        /// Returns the ID of the created proposal
        pub fn make_proposal(&mut self, temperature_check_id: u64) -> u64 {
            // Get the temperature check
            let mut tc = self
                .temperature_checks
                .get_mut(&temperature_check_id)
                .expect("Temperature check not found");

            assert!(
                tc.elevated_proposal_id.is_none(),
                "Temperature check has already been elevated to a proposal"
            );

            let proposal_id = self.proposal_count;
            self.proposal_count += 1;

            let now = Clock::current_time_rounded_to_seconds();
            let deadline = now.add_days(self.governance_parameters.proposal_length_days as i64).unwrap();

            let proposal = Proposal {
                title: tc.title.clone(),
                description: tc.description.clone(),
                vote_options: tc.vote_options.clone(),
                attachments: tc.attachments.clone(),
                rfc_url: tc.rfc_url.clone(),
                quorum: self.governance_parameters.proposal_quorum,
                votes: KeyValueStore::new(),
                approval_threshold: self.governance_parameters.proposal_approval_threshold,
                start: now,
                deadline,
                temperature_check_id,
            };

            tc.elevated_proposal_id = Some(proposal_id);
            drop(tc);

            self.proposals.insert(proposal_id, proposal);

            proposal_id
        }

        /// Vote on a temperature check
        /// The account must prove its presence
        pub fn vote_on_temperature_check(
            &mut self,
            account: Global<Account>,
            temperature_check_id: u64,
            vote: TemperatureCheckVote,
        ) {
            // Verify the account is present in the transaction
            Runtime::assert_access_rule(account.get_owner_role().rule);

            // Get the temperature check
            let mut tc = self
                .temperature_checks
                .get_mut(&temperature_check_id)
                .expect("Temperature check not found");

            // Check the vote is still open
            let now = Clock::current_time_rounded_to_seconds();
            assert!(
                now.compare(tc.start, TimeComparisonOperator::Gte),
                "Voting has not started yet"
            );
            assert!(
                now.compare(tc.deadline, TimeComparisonOperator::Lt),
                "Voting has ended"
            );

            // Check the account has not already voted
            assert!(
                tc.votes.get(&account).is_none(),
                "Account has already voted on this temperature check"
            );

            // Record the vote
            tc.votes.insert(account, vote);
        }

        /// Vote on a proposal
        /// The account must prove its presence
        pub fn vote_on_proposal(
            &mut self,
            account: Global<Account>,
            proposal_id: u64,
            vote: ProposalVoteOptionId,
        ) {
            // Verify the account is present in the transaction
            Runtime::assert_access_rule(account.get_owner_role().rule);

            // Get the proposal
            let mut proposal = self
                .proposals
                .get_mut(&proposal_id)
                .expect("Proposal not found");

            // Check the vote is still open
            let now = Clock::current_time_rounded_to_seconds();
            assert!(
                now.compare(proposal.start, TimeComparisonOperator::Gte),
                "Voting has not started yet"
            );
            assert!(
                now.compare(proposal.deadline, TimeComparisonOperator::Lt),
                "Voting has ended"
            );

            // Validate the vote option exists
            assert!(
                proposal.vote_options.iter().any(|opt| opt.id == vote),
                "Invalid vote option"
            );

            // Check the account has not already voted
            assert!(
                proposal.votes.get(&account).is_none(),
                "Account has already voted on this proposal"
            );

            // Record the vote
            proposal.votes.insert(account, vote);
        }

        /// Returns the current governance parameters
        pub fn get_governance_parameters(&self) -> GovernanceParameters {
            self.governance_parameters.clone()
        }

        /// Returns the current temperature check count
        pub fn get_temperature_check_count(&self) -> u64 {
            self.temperature_check_count
        }

        /// Returns the current proposal count
        pub fn get_proposal_count(&self) -> u64 {
            self.proposal_count
        }

        /// Updates the governance parameters (owner only)
        pub fn update_governance_parameters(&mut self, new_params: GovernanceParameters) {
            self.governance_parameters = new_params;
        }
    }
}
