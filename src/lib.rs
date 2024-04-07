mod donations;
use std::collections::HashMap;

use near_sdk::{near, AccountId, NearToken};

#[near(contract_state)]
pub struct Contract {
    owner: AccountId,
    donations: HashMap<AccountId, NearToken>,
}

impl Default for Contract {
    fn default() -> Self {
        Self {
            owner: "v2.faucet.nonofficial.testnet".parse().unwrap(),
            donations: HashMap::new(),
        }
    }
}

#[near]
impl Contract {
    #[init]
    #[private]
    pub fn init(owner: AccountId) -> Self {
        Self {
            owner,
            ..Default::default()
        }
    }

    // Public method
    pub fn get_owner(&self) -> &AccountId {
        &self.owner
    }

    #[private]
    pub fn change_owner(&mut self, new_owner: AccountId) {
        self.owner = new_owner
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::{test_utils::VMContextBuilder, testing_env, AccountId, NearToken};

    use crate::Contract;

    const OWNER: &str = "beneficiary";
    const ONE_NEAR: NearToken = NearToken::from_near(1);

    fn set_context(account_id: &str, amount: NearToken) {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(account_id.parse().unwrap());
        builder.attached_deposit(amount);

        testing_env!(builder.build())
    }

    #[test]
    fn init() {
        let contract = Contract::init(OWNER.parse().unwrap());
        assert_eq!(
            contract.owner,
            OWNER.parse::<AccountId>().unwrap().to_string()
        );
    }

    #[test]
    fn donate() {
        let mut contract = Contract::init(OWNER.parse().unwrap());

        set_context("alice", ONE_NEAR);
        contract.donate();
        let first_donation = contract.get_donation_for_account("alice".parse().unwrap());

        assert_eq!(first_donation.total_amount, ONE_NEAR);

        set_context("bob", ONE_NEAR.saturating_mul(2));
        contract.donate();
        let second_donation = contract.get_donation_for_account("bob".parse().unwrap());

        assert_eq!(second_donation.total_amount, ONE_NEAR.saturating_mul(2));

        // Alice donates again
        set_context("alice", ONE_NEAR);
        contract.donate();
        let first_donation = contract.get_donation_for_account("alice".parse().unwrap());

        assert_eq!(first_donation.total_amount, ONE_NEAR.saturating_mul(2));
        assert_eq!(contract.number_of_donors(), 2);
    }
}
