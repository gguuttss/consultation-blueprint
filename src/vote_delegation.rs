use scrypto::prelude::*;
use crate::Delegation;

#[blueprint]
mod vote_delegation {
    use super::*;

    enable_method_auth! {
        roles {
            owner => updatable_by: [];
        },
        methods {
            // Public methods
            make_delegation => PUBLIC;
            remove_delegation => PUBLIC;
            get_delegations => PUBLIC;
            get_delegatee_delegators => PUBLIC;
        }
    }

    struct VoteDelegation {
        /// Key: delegatee (person allowed to vote for others)
        /// Value: KVS of delegators using this delegatee, and the fraction of their power allocated
        pub delegatees: KeyValueStore<Global<Account>, KeyValueStore<Global<Account>, Decimal>>,

        /// Key: delegator (person that has delegated their voting power to another)
        /// Value: Delegation struct, holds all the user's delegations
        pub delegators: KeyValueStore<Global<Account>, Vec<Delegation>>,
    }

    impl VoteDelegation {
        /// Instantiates the vote delegation component with the given owner badge
        pub fn instantiate(owner_badge: ResourceAddress) -> Global<VoteDelegation> {
            Self {
                delegatees: KeyValueStore::new(),
                delegators: KeyValueStore::new(),
            }
            .instantiate()
            .prepare_to_globalize(OwnerRole::Fixed(rule!(require(owner_badge))))
            .roles(roles! {
                owner => rule!(require(owner_badge));
            })
            .globalize()
        }

        /// Delegate voting power from delegator to delegatee
        /// The delegator must prove their presence
        pub fn make_delegation(
            &mut self,
            delegator: Global<Account>,
            delegatee: Global<Account>,
            fraction: Decimal,
            valid_until: Instant,
        ) {
            // Verify the delegator is present in the transaction
            Runtime::assert_access_rule(delegator.get_owner_role().rule);

            // Validate inputs
            assert!(
                fraction > Decimal::ZERO && fraction <= Decimal::ONE,
                "Fraction must be between 0 (exclusive) and 1 (inclusive)"
            );
            assert!(
                delegator != delegatee,
                "Cannot delegate to yourself"
            );

            let now = Clock::current_time_rounded_to_seconds();
            assert!(
                valid_until.compare(now, TimeComparisonOperator::Gt),
                "Delegation must be valid for some time in the future"
            );

            // Check total delegation doesn't exceed 100%
            let mut total_delegated = Decimal::ZERO;
            if let Some(existing_delegations) = self.delegators.get(&delegator) {
                for delegation in existing_delegations.iter() {
                    // Only count delegations that are still valid
                    if delegation.valid_until.compare(now, TimeComparisonOperator::Gt) {
                        // Check if we're updating an existing delegation to the same delegatee
                        if delegation.delegatee == delegatee {
                            // This is an update, don't count the old one
                            continue;
                        }
                        total_delegated = total_delegated + delegation.fraction;
                    }
                }
            }
            assert!(
                total_delegated + fraction <= Decimal::ONE,
                "Total delegation cannot exceed 100%"
            );

            // Create the new delegation
            let new_delegation = Delegation {
                delegatee,
                fraction,
                valid_until,
            };

            // Update delegators map
            let has_existing = self.delegators.get(&delegator).is_some();
            if has_existing {
                let mut delegations = self.delegators.get_mut(&delegator).unwrap();
                // Remove any existing delegation to the same delegatee
                delegations.retain(|d| d.delegatee != delegatee);
                delegations.push(new_delegation);
            } else {
                self.delegators.insert(delegator, vec![new_delegation]);
            }

            // Update delegatees map
            let delegatee_exists = self.delegatees.get(&delegatee).is_some();
            if !delegatee_exists {
                self.delegatees.insert(delegatee, KeyValueStore::new());
            }
            let mut delegatee_map = self.delegatees.get_mut(&delegatee).unwrap();
            delegatee_map.insert(delegator, fraction);
        }

        /// Remove a delegation from delegator to delegatee
        /// The delegator must prove their presence
        pub fn remove_delegation(
            &mut self,
            delegator: Global<Account>,
            delegatee: Global<Account>,
        ) {
            // Verify the delegator is present in the transaction
            Runtime::assert_access_rule(delegator.get_owner_role().rule);

            // Remove from delegators map
            if let Some(mut delegations) = self.delegators.get_mut(&delegator) {
                let initial_len = delegations.len();
                delegations.retain(|d| d.delegatee != delegatee);
                assert!(
                    delegations.len() < initial_len,
                    "No delegation found to the specified delegatee"
                );
            } else {
                panic!("No delegations found for this account");
            }

            // Remove from delegatees map
            if let Some(mut delegatee_map) = self.delegatees.get_mut(&delegatee) {
                delegatee_map.remove(&delegator);
            }
        }

        /// Get all delegations made by a delegator
        pub fn get_delegations(&self, delegator: Global<Account>) -> Vec<Delegation> {
            self.delegators
                .get(&delegator)
                .map(|d| d.clone())
                .unwrap_or_default()
        }

        /// Get the fraction delegated to a delegatee from a specific delegator
        pub fn get_delegatee_delegators(
            &self,
            delegatee: Global<Account>,
            delegator: Global<Account>,
        ) -> Option<Decimal> {
            self.delegatees
                .get(&delegatee)
                .and_then(|m| m.get(&delegator).map(|d| *d))
        }
    }
}
