use crate::Contract;
use crate::ContractExt;
use near_sdk::log;
use near_sdk::Promise;
use near_sdk::{env, near, require, AccountId, NearToken};

pub const STORAGE_COST: NearToken = NearToken::from_millinear(1);

#[near(serializers = [borsh, json])]
pub struct Donation {
    pub account_id: AccountId,
    pub total_amount: NearToken,
}

#[near]
impl Contract {
    #[payable]
    pub fn donate(&mut self) -> String {
        let donor = env::predecessor_account_id();
        let donation_amount = env::attached_deposit();

        require!(
            donation_amount > STORAGE_COST,
            format!("Attach at least {STORAGE_COST} yoctoNEAR to cover for the storage cost")
        );

        let mut donated_so_far = *self
            .donations
            .get(&donor)
            .unwrap_or(&NearToken::from_near(0));

        let to_transfer = if donated_so_far.is_zero() {
            donation_amount.saturating_sub(STORAGE_COST).to_owned()
        } else {
            donation_amount
        };

        donated_so_far = donated_so_far.saturating_add(donation_amount);

        log!(
            format!("Thank you {donor} for donating {donation_amount}! You donated a total of {donated_so_far}")
        );

        self.donations.insert(donor, donated_so_far);

        Promise::new(self.owner.clone()).transfer(to_transfer);

        donated_so_far.to_string()
    }

    pub fn get_donation_for_account(&self, account_id: AccountId) -> Donation {
        Donation {
            total_amount: *self
                .donations
                .get(&account_id)
                .unwrap_or(&NearToken::from_near(0)),
            account_id,
        }
    }

    pub fn number_of_donors(&self) -> usize {
        self.donations.len()
    }

    pub fn get_donations(&self, from_index: Option<u64>, limit: Option<u64>) -> Vec<Donation> {
        self.donations
            .iter()
            .skip(from_index.unwrap_or(0) as usize)
            .take(limit.unwrap_or(10) as usize)
            .map(|(account_id, total_amount)| Donation {
                account_id: account_id.clone(),
                total_amount: total_amount.clone(),
            })
            .collect()
    }
}
