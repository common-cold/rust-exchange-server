use std::str::FromStr;

use bigdecimal::BigDecimal;
use common::OnRampArgs;

use crate::TestHarness;



#[tokio::test]
pub async fn onramp_inr_eur_usd_on_fresh_users() {
    let harness = TestHarness::start().await;

    //inr
    let email_inr = String::from("user@gmail.com");
    let pass_inr = String::from("password");
    let user_inr = harness.create_user_in_db(&email_inr, &pass_inr).await;

    //eur
    let email_eur = String::from("user1@gmail.com");
    let pass_eur= String::from("password");
    let user_eur= harness.create_user_in_db(&email_eur, &pass_eur).await;

    //usd
    let email_usd = String::from("user2@gmail.com");
    let pass_usd= String::from("password");
    let user_usd= harness.create_user_in_db(&email_usd, &pass_usd).await;

    
    let onramp_inr_args = OnRampArgs {
        user_id: user_inr.id,
        currency: common::Currency::INR,
        amount: BigDecimal::from_str("20").unwrap(),
        usdc_conversion_rate: None
    };

    let onramp_eur_args = OnRampArgs {
        user_id: user_eur.id,
        currency: common::Currency::EUR,
        amount: BigDecimal::from_str("20").unwrap(),
        usdc_conversion_rate: None
    };

    let onramp_usd_args = OnRampArgs {
        user_id: user_usd.id,
        currency: common::Currency::USD,
        amount: BigDecimal::from_str("20").unwrap(),
        usdc_conversion_rate: None
    };

    harness.onramp_balance(onramp_inr_args.clone()).await;
    harness.onramp_balance(onramp_eur_args.clone()).await;
    harness.onramp_balance(onramp_usd_args.clone()).await;
    harness.flush().await.unwrap();

    let rate_inr = harness.get_conversion_rate(common::Currency::INR).await;
    let inr_as_usdc_amount_in_base_units = harness.calculate_usdc_base_units(
        onramp_inr_args.amount, 
        BigDecimal::from_str(&rate_inr.to_string()).unwrap()
    );

    let rate_eur = harness.get_conversion_rate(common::Currency::EUR).await;
    let eur_as_usdc_amount_in_base_units = harness.calculate_usdc_base_units(
        onramp_eur_args.amount, 
        BigDecimal::from_str(&rate_eur.to_string()).unwrap()
    );

    let rate_usd = harness.get_conversion_rate(common::Currency::USD).await;
    let usd_as_usdc_amount_in_base_units = harness.calculate_usdc_base_units(
        onramp_usd_args.amount, 
        BigDecimal::from_str(&rate_usd.to_string()).unwrap()
    );

    //check balances in db
    let inr_balance = harness.get_balance_from_db(user_inr.id).await;
    let eur_balance = harness.get_balance_from_db(user_eur.id).await;
    let usd_balance = harness.get_balance_from_db(user_usd.id).await;

    assert_eq!(inr_balance.free_quote_qty, inr_as_usdc_amount_in_base_units);
    assert_eq!(inr_balance.free_base_qty, BigDecimal::from(0));
    assert_eq!(inr_balance.locked_base_qty, BigDecimal::from(0));
    assert_eq!(inr_balance.locked_quote_qty, BigDecimal::from(0));

    assert_eq!(eur_balance.free_quote_qty, eur_as_usdc_amount_in_base_units);
    assert_eq!(eur_balance.free_base_qty, BigDecimal::from(0));
    assert_eq!(eur_balance.locked_base_qty, BigDecimal::from(0));
    assert_eq!(eur_balance.locked_quote_qty, BigDecimal::from(0));

    assert_eq!(usd_balance.free_quote_qty, usd_as_usdc_amount_in_base_units);
    assert_eq!(usd_balance.free_base_qty, BigDecimal::from(0));
    assert_eq!(usd_balance.locked_base_qty, BigDecimal::from(0));
    assert_eq!(usd_balance.locked_quote_qty, BigDecimal::from(0));
}