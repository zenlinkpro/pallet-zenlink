use crate::{mock::*, Error, SwapHandler};
use frame_support::{
    assert_noop, assert_ok,
};

const TEST_TOKEN: &AssetInfo = &AssetInfo {
    name: *b"zenlinktesttoken",
    symbol: *b"TEST____",
    decimals: 0u8,
};

const TEST_OTHER_TOKEN: &AssetInfo = &AssetInfo {
    name: *b"zenlinktesttoken",
    symbol: *b"TEST2___",
    decimals: 0u8,
};

const ALICE: u128 = 1;
const BOB: u128 = 2;
const CHAREL: u128 = 3;

// u128 for MODULE_ID.into_sub_account()
const EXCHANGE_ACCOUNT: u128 = 15310315390164549602772283245;
const EXCHANGE_ACCOUNT2: u128 = 94538477904428887196316233581;

#[test]
fn issuing_asset_units_to_issuer_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(Currency::free_balance(&ALICE), 10000);
        assert_eq!(TokenModule::inner_issue(&ALICE, 100, TEST_TOKEN), 0);
        assert_eq!(TokenModule::balance_of(&0, &ALICE), 100);
        assert_eq!(TokenModule::asset_info(&0), Some(TEST_TOKEN.clone()));
        assert_eq!(Currency::free_balance(&ALICE), 10000);
    });
}

#[test]
fn create_exchange_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(TokenModule::inner_issue(&ALICE, 10000, TEST_TOKEN), 0);

        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));

        assert_eq!(
            DexModule::get_exchange_id(&SwapHandler::from_exchange_id(0)).unwrap(),
            0
        );
        assert_eq!(
            DexModule::get_exchange_id(&SwapHandler::from_asset_id(1)).is_err(),
            true
        );

        assert_eq!(DexModule::get_exchange_info(0).unwrap().token_id, 0);
        assert_eq!(DexModule::get_exchange_info(0).unwrap().liquidity_id, 1);
        assert_eq!(
            DexModule::get_exchange_info(0).unwrap().account,
            EXCHANGE_ACCOUNT
        );
        assert_eq!(TokenModule::balance_of(&0, &EXCHANGE_ACCOUNT), 0);
        assert_eq!(TokenModule::balance_of(&1, &EXCHANGE_ACCOUNT), 0);
        assert_eq!(TokenModule::total_supply(&1), 0);
    });
}

#[test]
fn create_exchange_should_not_work() {
    new_test_ext().execute_with(|| {
        assert_noop!(
            DexModule::create_exchange(Origin::signed(ALICE), 0),
            Error::<Test>::TokenNotExists
        );

        assert_eq!(TokenModule::inner_issue(&ALICE, 10000, TEST_TOKEN), 0);
        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));
        assert_noop!(
            DexModule::create_exchange(Origin::signed(ALICE), 0),
            Error::<Test>::ExchangeAlreadyExists
        );

        assert_noop!(
            DexModule::create_exchange(Origin::signed(ALICE), 1),
            Error::<Test>::DeniedSwap
        );
    })
}

#[test]
fn create_more_exchanges_should_work() {
    new_test_ext().execute_with(|| {
        // create TEST_TOKEN exchange
        assert_eq!(TokenModule::inner_issue(&ALICE, 42, TEST_TOKEN), 0);
        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));

        assert_eq!(
            DexModule::get_exchange_info(0).unwrap().account,
            EXCHANGE_ACCOUNT
        );
        assert_eq!(
            DexModule::get_exchange_info(0).unwrap().token_id,
            0
        );
        assert_eq!(
            DexModule::get_exchange_info(0).unwrap().liquidity_id,
            1
        );

        assert_ok!(TokenModule::inner_approve(
            &0,
            &ALICE,
            &EXCHANGE_ACCOUNT,
            42
        ));

        assert_eq!(TokenModule::total_supply(&1), 0);
        // Add 420 currency and 42 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            420,
            0,
            42,
            100
        ));
        assert_eq!(TokenModule::total_supply(&1), 420);


        // create TEST_OTHER_TOKEN exchange
        assert_eq!(TokenModule::inner_issue(&BOB, 42, TEST_OTHER_TOKEN), 2);
        assert_ok!(DexModule::create_exchange(Origin::signed(BOB), 2));

        assert_eq!(
            DexModule::get_exchange_info(1).unwrap().account,
            EXCHANGE_ACCOUNT2
        );
        assert_eq!(
            DexModule::get_exchange_info(1).unwrap().token_id,
            2
        );
        assert_eq!(
            DexModule::get_exchange_info(1).unwrap().liquidity_id,
            3
        );

        assert_ok!(TokenModule::inner_approve(
            &2,
            &BOB,
            &EXCHANGE_ACCOUNT2,
            42
        ));

        assert_eq!(TokenModule::total_supply(&3), 0);
        // Add 420 currency and 42 other token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(BOB),
            SwapHandler::from_exchange_id(1),
            420,
            0,
            42,
            100
        ));
        assert_eq!(TokenModule::total_supply(&3), 420);
    })
}

#[test]
fn add_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        // Initial currency 10000
        assert_eq!(Currency::free_balance(&ALICE), 10000);

        // The asset_id = 0
        assert_eq!(TokenModule::inner_issue(&ALICE, 5000, TEST_TOKEN), 0);
        assert_eq!(TokenModule::balance_of(&0, &ALICE), 5000);

        // The exchange_id = 0, one liquidity token asset_id = 1
        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));
        assert_eq!(
            DexModule::get_exchange_info(0).unwrap().account,
            EXCHANGE_ACCOUNT
        );

        // Alice approve 1000 token for EXCHANGE_ACCOUNT
        assert_ok!(TokenModule::inner_approve(
            &0,
            &ALICE,
            &EXCHANGE_ACCOUNT,
            1000
        ));

        // Exchange 0 liquidity is 0
        assert_eq!(TokenModule::total_supply(&1), 0);

        // Some no-ops
        // (1) ExchangeNotExists
        assert_noop!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(1),   // no exchange
            100,
            0,
            1000,
            100
        ), Error::<Test>::ExchangeNotExists);

        // (2) ZeroCurrency
        assert_noop!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            0,    // zero currency
            0,
            1000,
            100
        ), Error::<Test>::ZeroCurrency);

        // (3) ZeroToken
        assert_noop!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            0,
            0,   // zero token
            100,
        ), Error::<Test>::ZeroToken);

        // (4) Deadline
        assert_noop!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            0,
            1000,
            0  // deadline
        ), Error::<Test>::Deadline);

        // Add 1000 currency and 100 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            0,
            1000,
            100
        ));

        // Total supply liquidity is 100
        assert_eq!(TokenModule::total_supply(&1), 100);
        assert_eq!(TokenModule::balance_of(&1, &ALICE), 100);
        assert_eq!(TokenModule::balance_of(&1, &EXCHANGE_ACCOUNT), 0);

        // The balances of currency
        assert_eq!(Currency::free_balance(&ALICE), 10000 - 100);
        assert_eq!(Currency::free_balance(&EXCHANGE_ACCOUNT), 100);

        // The token balances
        assert_eq!(TokenModule::balance_of(&0, &ALICE), 5000 - 1000);
        assert_eq!(TokenModule::balance_of(&0, &EXCHANGE_ACCOUNT), 1000);

        // (5) RequestedZeroLiquidity
        assert_noop!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            0,  // only liquidity zero
            1000,
            100
        ), Error::<Test>::RequestedZeroLiquidity);

        // (6) AllowanceLow
        assert_noop!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            1,
            1000,
            100
        ), Error::<Test>::AllowanceLow);

        // Alice approve 1000 token for EXCHANGE_ACCOUNT
        assert_ok!(TokenModule::inner_approve(
            &0,
            &ALICE,
            &EXCHANGE_ACCOUNT,
            1000
        ));

        // (7) TooLowLiquidity
        assert_noop!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            101,    // min liquidity is set too high
            1000,
            100
        ), Error::<Test>::TooLowLiquidity);

        // (7) TooManyToken
        assert_noop!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            1,
            1,  // max token is set too low
            100
        ), Error::<Test>::TooManyToken);

        // again Add 1000 currency and 100 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            100,
            1000,
            100
        ));

        // Total supply liquidity is 100
        assert_eq!(TokenModule::total_supply(&1), 200);
        assert_eq!(TokenModule::balance_of(&1, &ALICE), 200);
        assert_eq!(TokenModule::balance_of(&1, &EXCHANGE_ACCOUNT), 0);

        // The balances of currency
        assert_eq!(Currency::free_balance(&ALICE), 10000 - 200);
        assert_eq!(Currency::free_balance(&EXCHANGE_ACCOUNT), 200);

        // The token balances
        assert_eq!(TokenModule::balance_of(&0, &ALICE), 5000 - 2000);
        assert_eq!(TokenModule::balance_of(&0, &EXCHANGE_ACCOUNT), 2000);
    })
}

#[test]
fn remove_liquidity_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(TokenModule::inner_issue(&ALICE, 5000, TEST_TOKEN), 0);
        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));
        assert_ok!(TokenModule::inner_approve(
            &0,
            &ALICE,
            &EXCHANGE_ACCOUNT,
            1000
        ));

        // Add 100 currency and 500 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            0,
            500,
            100
        ));

        // Total supply liquidity is 100
        assert_eq!(TokenModule::total_supply(&1), 100);
        assert_eq!(TokenModule::balance_of(&1, &ALICE), 100);

        // Some no-ops
        // (1) ExchangeNotExists
        assert_noop!(DexModule::remove_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(1),   // no exchange
            100,
            100,
            500,
            100
        ), Error::<Test>::ExchangeNotExists);

        // (2) BurnZeroZLKShares
        assert_noop!(DexModule::remove_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            0,  // zero zlk_to_burn
            100,
            500,
            100
        ), Error::<Test>::BurnZeroZLKShares);

        // (3) NotEnoughCurrency
        assert_noop!(DexModule::remove_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            1000,    // min_currency
            500,
            100
        ), Error::<Test>::NotEnoughCurrency);

        // (4) NotEnoughToken
        assert_noop!(DexModule::remove_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            100,
            5000,    // min_token
            100
        ), Error::<Test>::NotEnoughToken);

        // (5) Deadline
        assert_noop!(DexModule::remove_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            100,
            500,
            0   // deadline
        ), Error::<Test>::Deadline);

        assert_ok!(DexModule::remove_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            100,
            500,
            100
        ));

        assert_eq!(TokenModule::total_supply(&1), 0);

        // Fresh exchange with no liquidity
        // Add 100 currency and 500 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            100,
            0,
            500,
            100
        ));

        assert_eq!(TokenModule::total_supply(&1), 100);

    })
}

#[test]
fn currency_to_token_input_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(TokenModule::inner_issue(&ALICE, 42, TEST_TOKEN), 0);
        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));
        assert_ok!(TokenModule::inner_approve(
            &0,
            &ALICE,
            &EXCHANGE_ACCOUNT,
            42
        ));

        // Add 420 currency and 42 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            420,
            0,
            42,
            100
        ));

        // Total supply liquidity is 420
        assert_eq!(TokenModule::total_supply(&1), 420);

        assert_noop!(DexModule::currency_to_token_input(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            300,
            20,     // min tokens is set too high
            100,
            ALICE
        ), Error::<Test>::NotEnoughToken);

        assert_eq!(Currency::free_balance(ALICE), 10000 - 420);
        assert_eq!(Currency::free_balance(EXCHANGE_ACCOUNT), 420);
        assert_eq!(TokenModule::balance_of(&0,&BOB), 0);
        assert_eq!(TokenModule::balance_of(&0,&EXCHANGE_ACCOUNT), 42);

        assert_ok!(DexModule::currency_to_token_input(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            300,
            1,
            100,
            BOB
        ));

        assert_eq!(Currency::free_balance(ALICE), 10000 - 420 - 300);
        assert_eq!(Currency::free_balance(EXCHANGE_ACCOUNT), 420 + 300);
        assert_eq!(TokenModule::balance_of(&0,&EXCHANGE_ACCOUNT), 42 - 17);
        assert_eq!(TokenModule::balance_of(&0,&BOB), 17);
        assert_eq!(TokenModule::balance_of(&0,&ALICE), 0);

        // ZLK liquidity share should not change
        assert_eq!(TokenModule::balance_of(&1,&ALICE), 420);
    })
}

#[test]
fn currency_to_token_output_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(TokenModule::inner_issue(&ALICE, 42, TEST_TOKEN), 0);
        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));
        assert_ok!(TokenModule::inner_approve(
            &0,
            &ALICE,
            &EXCHANGE_ACCOUNT,
            42
        ));

        // Add 420 currency and 42 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_asset_id(0),
            420,
            0,
            42,
            100
        ));

        // Total supply liquidity is 420
        assert_eq!(TokenModule::total_supply(&1), 420);

        assert_noop!(DexModule::currency_to_token_output(
            Origin::signed(ALICE),
            SwapHandler::from_asset_id(0),
            17,
            200,     // max currency set too low for this token amount
            100,
            ALICE
        ), Error::<Test>::TooExpensiveCurrency);

        assert_ok!(DexModule::currency_to_token_output(
            Origin::signed(ALICE),
            SwapHandler::from_asset_id(0),
            17,
            300,     // just ok
            100,
            BOB
        ));

        assert_eq!(Currency::free_balance(ALICE), 10000 - 420 - 287);
        assert_eq!(Currency::free_balance(EXCHANGE_ACCOUNT), 420 + 287);
        assert_eq!(TokenModule::balance_of(&0,&EXCHANGE_ACCOUNT), 42 - 17);
        assert_eq!(TokenModule::balance_of(&0,&BOB), 17);
        assert_eq!(TokenModule::balance_of(&0,&ALICE), 0);

        // ZLK liquidity share should not change
        assert_eq!(TokenModule::balance_of(&1,&ALICE), 420);
    })
}

#[test]
fn token_to_currency_input_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(TokenModule::inner_issue(&ALICE, 42*2, TEST_TOKEN), 0);
        assert_ok!(TokenModule::inner_transfer(&0, &ALICE, &BOB, 42));
        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));
        assert_ok!(TokenModule::inner_approve(
            &0,
            &ALICE,
            &EXCHANGE_ACCOUNT,
            42
        ));

        // Add 420 currency and 42 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            420,
            0,
            42,
            100
        ));

        // Total supply liquidity is 420
        assert_eq!(TokenModule::total_supply(&1), 420);

        assert_ok!(TokenModule::inner_approve(
            &0,
            &BOB,
            &EXCHANGE_ACCOUNT,
            42
        ));

        assert_noop!(DexModule::token_to_currency_input(
            Origin::signed(BOB),
            SwapHandler::from_exchange_id(0),
            20,
            1000,   // min currency set too high for this token amount
            100,
            BOB
        ), Error::<Test>::NotEnoughCurrency);

        assert_ok!(DexModule::token_to_currency_input(
            Origin::signed(BOB),
            SwapHandler::from_exchange_id(0),
            20,
            1,
            100,
            BOB
        ));

        assert_eq!(Currency::free_balance(BOB), 10000 + 135);
        assert_eq!(Currency::free_balance(EXCHANGE_ACCOUNT), 420 - 135);
        assert_eq!(TokenModule::balance_of(&0,&EXCHANGE_ACCOUNT), 42 + 20);
        assert_eq!(TokenModule::balance_of(&0,&BOB), 42 - 20);
        assert_eq!(TokenModule::balance_of(&0,&ALICE), 0);

        // ZLK liquidity share should not change
        assert_eq!(TokenModule::balance_of(&1,&ALICE), 420);
    })
}

#[test]
fn token_to_currency_output_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(TokenModule::inner_issue(&ALICE, 42*2, TEST_TOKEN), 0);
        assert_ok!(TokenModule::inner_transfer(&0, &ALICE, &BOB, 42));
        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));
        assert_ok!(TokenModule::inner_approve(
            &0,
            &ALICE,
            &EXCHANGE_ACCOUNT,
            42
        ));

        // Add 420 currency and 42 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            420,
            0,
            42,
            100
        ));

        // Total supply liquidity is 420
        assert_eq!(TokenModule::total_supply(&1), 420);

        assert_ok!(TokenModule::inner_approve(
            &0,
            &BOB,
            &EXCHANGE_ACCOUNT,
            42
        ));

        assert_noop!(DexModule::token_to_currency_output(
            Origin::signed(BOB),
            SwapHandler::from_asset_id(0),
            135,
            1,   // max token set too low for this currency
            100,
            BOB
        ), Error::<Test>::TooExpensiveToken);

        assert_ok!(DexModule::token_to_currency_output(
            Origin::signed(BOB),
            SwapHandler::from_asset_id(0),
            135,
            1000,
            100,
            BOB
        ));

        assert_eq!(Currency::free_balance(BOB), 10000 + 135);
        assert_eq!(Currency::free_balance(EXCHANGE_ACCOUNT), 420 - 135);
        assert_eq!(TokenModule::balance_of(&0,&EXCHANGE_ACCOUNT), 42 + 20);
        assert_eq!(TokenModule::balance_of(&0,&BOB), 42 - 20);
        assert_eq!(TokenModule::balance_of(&0,&ALICE), 0);

        // ZLK liquidity share should not change
        assert_eq!(TokenModule::balance_of(&1,&ALICE), 420);
    })
}

#[test]
fn token_to_token_input_should_work() {
    new_test_ext().execute_with(|| {
        // create TEST_TOKEN exchange
        assert_eq!(TokenModule::inner_issue(&ALICE, 42+42, TEST_TOKEN), 0);
        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));
        assert_ok!(TokenModule::inner_approve(
            &0,
            &ALICE,
            &EXCHANGE_ACCOUNT,
            42
        ));
        // Add 420 currency and 42 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            420,
            0,
            42,
            100
        ));
        assert_eq!(TokenModule::total_supply(&1), 420);

        // create TEST_OTHER_TOKEN exchange
        assert_eq!(TokenModule::inner_issue(&BOB, 42, TEST_OTHER_TOKEN), 2);
        assert_ok!(DexModule::create_exchange(Origin::signed(BOB), 2));
        assert_ok!(TokenModule::inner_approve(
            &2,
            &BOB,
            &EXCHANGE_ACCOUNT2,
            42
        ));
        // Add 420 currency and 42 other token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(BOB),
            SwapHandler::from_asset_id(2),
            420,
            0,
            42,
            100
        ));
        assert_eq!(TokenModule::total_supply(&3), 420);

        // Transfer 42 token to CHAREL
        assert_ok!(TokenModule::inner_transfer(&0, &ALICE, &CHAREL, 42));

        assert_eq!(TokenModule::balance_of(&0, &CHAREL), 42);
        assert_eq!(TokenModule::balance_of(&2, &CHAREL), 0);

        assert_ok!(TokenModule::inner_approve(
            &0,
            &CHAREL,
            &EXCHANGE_ACCOUNT,
            42
        ));

        assert_noop!(DexModule::token_to_token_input(
            Origin::signed(CHAREL),
            SwapHandler::from_asset_id(0),
            SwapHandler::from_exchange_id(1),
            42,
            100,     //min other token set too high
            100,
            CHAREL
        ), Error::<Test>::NotEnoughToken);

        assert_ok!(DexModule::token_to_token_input(
            Origin::signed(CHAREL),
            SwapHandler::from_asset_id(0),
            SwapHandler::from_exchange_id(1),
            42,
            1,
            100,
            CHAREL
        ));

        assert_eq!(TokenModule::balance_of(&0, &CHAREL), 0);
        assert_eq!(TokenModule::balance_of(&2, &CHAREL), 13);

        assert_eq!(TokenModule::total_supply(&1), 420);
        assert_eq!(TokenModule::total_supply(&3), 420);
    })
}

#[test]
fn token_to_token_output_should_work() {
    new_test_ext().execute_with(|| {
        // create TEST_TOKEN exchange
        assert_eq!(TokenModule::inner_issue(&ALICE, 42+42, TEST_TOKEN), 0);
        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));
        assert_ok!(TokenModule::inner_approve(
            &0,
            &ALICE,
            &EXCHANGE_ACCOUNT,
            42
        ));
        // Add 420 currency and 42 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            420,
            0,
            42,
            100
        ));
        assert_eq!(TokenModule::total_supply(&1), 420);

        // create TEST_OTHER_TOKEN exchange
        assert_eq!(TokenModule::inner_issue(&BOB, 42, TEST_OTHER_TOKEN), 2);
        assert_ok!(DexModule::create_exchange(Origin::signed(BOB), 2));
        assert_ok!(TokenModule::inner_approve(
            &2,
            &BOB,
            &EXCHANGE_ACCOUNT2,
            42
        ));
        // Add 420 currency and 42 other token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(BOB),
            SwapHandler::from_exchange_id(1),
            420,
            0,
            42,
            100
        ));
        assert_eq!(TokenModule::total_supply(&3), 420);

        // Transfer 42 token to CHAREL
        assert_ok!(TokenModule::inner_transfer(&0, &ALICE, &CHAREL, 42));

        assert_eq!(TokenModule::balance_of(&0, &CHAREL), 42);
        assert_eq!(TokenModule::balance_of(&2, &CHAREL), 0);

        assert_ok!(TokenModule::inner_approve(
            &0,
            &CHAREL,
            &EXCHANGE_ACCOUNT,
            42
        ));

        assert_noop!(DexModule::token_to_token_output(
            Origin::signed(CHAREL),
            SwapHandler::from_exchange_id(0),
            SwapHandler::from_asset_id(2),
            13,
            1,     //max token set too low
            100,
            CHAREL
        ), Error::<Test>::TooExpensiveToken);

        assert_ok!(DexModule::token_to_token_output(
            Origin::signed(CHAREL),
            SwapHandler::from_exchange_id(0),
            SwapHandler::from_asset_id(2),
            13,
            42,
            100,
            CHAREL
        ));

        assert_eq!(TokenModule::balance_of(&0, &CHAREL), 7);
        assert_eq!(TokenModule::balance_of(&2, &CHAREL), 13);

        assert_eq!(TokenModule::total_supply(&1), 420);
        assert_eq!(TokenModule::total_supply(&3), 420);
    })
}

#[test]
fn zlk_liquidity_transfer_and_remove_should_work() {
    new_test_ext().execute_with(|| {
        assert_eq!(TokenModule::inner_issue(&ALICE, 42, TEST_TOKEN), 0);
        assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));
        assert_ok!(TokenModule::inner_approve(
            &0,
            &ALICE,
            &EXCHANGE_ACCOUNT,
            42
        ));

        // Add 420 currency and 42 token
        assert_ok!(DexModule::add_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            420,
            0,
            42,
            100
        ));

        // Total supply liquidity is 420
        assert_eq!(TokenModule::total_supply(&1), 420);
        assert_eq!(TokenModule::balance_of(&1, &EXCHANGE_ACCOUNT), 0);
        assert_eq!(TokenModule::balance_of(&0, &EXCHANGE_ACCOUNT), 42);
        assert_eq!(TokenModule::balance_of(&1, &ALICE), 420);
        assert_eq!(Currency::free_balance(ALICE), 10000-420);
        // Transfer ZLK liquidity token
        assert_ok!(TokenModule::inner_transfer(&1, &ALICE, &BOB, 100));
        assert_ok!(TokenModule::inner_transfer(&1, &ALICE, &CHAREL, 100));

        assert_ok!(DexModule::remove_liquidity(
            Origin::signed(BOB),
            SwapHandler::from_exchange_id(0),
            100,
            1,
            1,
            100
        ));

        assert_ok!(DexModule::remove_liquidity(
            Origin::signed(CHAREL),
            SwapHandler::from_exchange_id(0),
            100,
            1,
            1,
            100
        ));

        assert_ok!(DexModule::remove_liquidity(
            Origin::signed(ALICE),
            SwapHandler::from_exchange_id(0),
            420-100-100,
            1,
            1,
            100
        ));

        assert_eq!(TokenModule::balance_of(&0, &BOB), 10);
        assert_eq!(Currency::free_balance(BOB), 10000+100);

        assert_eq!(TokenModule::balance_of(&0, &CHAREL), 10);
        assert_eq!(Currency::free_balance(CHAREL), 10000+100);

        assert_eq!(TokenModule::balance_of(&0, &ALICE), 22);
        assert_eq!(Currency::free_balance(ALICE), 10000-420+220);

        assert_eq!(TokenModule::total_supply(&1), 0);
        assert_eq!(TokenModule::balance_of(&1, &ALICE), 0);
    })
}