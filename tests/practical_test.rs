use near_sdk::{near, AccountId, NearToken};
use serde_json::json;

const ONE_NEAR: NearToken = NearToken::from_near(1);
const STORAGE_COST: NearToken = NearToken::from_millinear(1);

#[tokio::test]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let sandbox = near_workspaces::sandbox().await?;
    let contract_wasm = near_workspaces::compile_project("./").await?;

    let contract = sandbox.dev_deploy(&contract_wasm).await?;
    let alice = sandbox.dev_create_account().await?;
    let bob = sandbox.dev_create_account().await?;
    let owner = sandbox.dev_create_account().await?;

    let owner_balance = owner.view_account().await?.balance;

    let init_result = contract
        .call("init")
        .args_json(json!({"owner": owner.id()}))
        .transact()
        .await?;

    assert!(init_result.is_success());

    let alice_first_donation_result = alice
        .call(contract.id(), "donate")
        .args_json(json!({}))
        .deposit(ONE_NEAR)
        .transact()
        .await?;

    assert!(alice_first_donation_result.is_success());

    let bob_first_donation_result = bob
        .call(contract.id(), "donate")
        .args_json(json!({}))
        .deposit(ONE_NEAR)
        .transact()
        .await?;

    assert!(bob_first_donation_result.is_success());

    let _ = alice
        .call(contract.id(), "donate")
        .args_json(json!({}))
        .deposit(ONE_NEAR.saturating_mul(3))
        .transact()
        .await?
        .into_result()?;

    let number_of_donors: usize = contract
        .view("number_of_donors")
        .args_json(json!({}))
        .await?
        .json()?;

    assert_eq!(number_of_donors, 2);

    #[derive(Debug, PartialEq)]
    #[near(serializers = [json])]
    struct Donation {
        account_id: AccountId,
        total_amount: NearToken,
    }

    let donation: Donation = contract
        .view("get_donation_for_account")
        .args_json(json!({"account_id": alice.id()}))
        .await?
        .json()?;

    assert_eq!(donation.total_amount, NearToken::from_near(4));

    let donations: Vec<Donation> = contract
        .view("get_donations")
        .args_json(json!({}))
        .await?
        .json()?;

    assert_eq!(
        donations,
        vec![
            Donation {
                account_id: alice.id().clone(),
                total_amount: NearToken::from_near(4),
            },
            Donation {
                account_id: bob.id().clone(),
                total_amount: NearToken::from_near(1),
            }
        ]
    );

    let donation_amount = NearToken::from_near(5).saturating_sub(STORAGE_COST.saturating_mul(2));
    let expected_balance = owner_balance.saturating_add(donation_amount);

    assert_eq!(owner.view_account().await?.balance, expected_balance);

    Ok(())
}
