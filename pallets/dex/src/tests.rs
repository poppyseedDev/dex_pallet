use crate::{mock::*, Error, Event, *};
use frame_support::{assert_noop, assert_ok, traits::fungible::Inspect};

use frame_support::traits::fungibles::Inspect as FSInspect;
//use sp_runtime::traits::BadOrigin;

type Balance = <Test as crate::Config>::NativeBalance;
type Assets = <Test as crate::Config>::Fungibles;

fn create_multiple_accounts() -> Vec<u64> {
	let mut accounts = Vec::new();
	for i in 0..10 {
		let account_id = i as u64 + 1;
		assert_ok!(Dex::setup_account(account_id, 1_000_000, vec![(1, 100), (2, 200), (3, 300)]));
		accounts.push(account_id);
	}
	accounts
}

#[test]
fn test_create_multiple_accounts() {
	new_test_ext().execute_with(|| {
		let accounts = create_multiple_accounts();
		assert_eq!(accounts.len(), 10);
	});
}

#[test]
fn test_setup_function() {
	new_test_ext().execute_with(|| {
		assert_eq!(Balance::total_balance(&1), 0);

		assert_ok!(Dex::setup_account(1, 1_000_000, vec![(1, 100), (2, 200), (3, 300)]));

		assert_eq!(Balance::total_balance(&1), 1_000_000);

		assert_eq!(Assets::total_balance(0, &1), 0);
		assert_eq!(Assets::total_balance(1, &1), 100);
		assert_eq!(Assets::total_balance(2, &1), 200);
		assert_eq!(Assets::total_balance(3, &1), 300);

		assert_ok!(Dex::setup_account(2, 1_000_000, vec![(1, 100), (2, 200), (3, 300)]));
	});
}

#[test]
fn create_pool() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::setup_account(1, 1_000_000, vec![(1, 1_000), (2, 1_000), (3, 1_000)]));

		// checks if it can create a pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 100, 200));
		assert_eq!(Assets::total_balance(1, &1), 900);
		assert_eq!(Assets::total_balance(2, &1), 800);

		// pool already exists you should not be able to create another one
		assert_noop!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 100, 100), Error::<Test>::PoolAlreadyExists);
	});
}

#[test]
fn create_pool_and_add_more_liquidity() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::setup_account(1, 1_000_000, vec![(1, 1_000), (2, 1_000), (3, 1_000)]));

		// checks if it can create a pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 100, 200));
		assert_eq!(Assets::total_balance(1, &1), 900);
		assert_eq!(Assets::total_balance(2, &1), 800);

		// setup second account
		assert_ok!(Dex::setup_account(2, 1_000_000, vec![(1, 1_000), (2, 1_000), (3, 1_000)]));

		// add more liquidity
		assert_ok!(Dex::add_liquidity(RuntimeOrigin::signed(2), 1, 2, 100, 200));
		assert_eq!(Assets::total_balance(1, &2), 900);
		assert_eq!(Assets::total_balance(2, &2), 800);

	});
}

#[test]
fn create_pool_and_remove_some_liquidity() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::setup_account(1, 1_000_000, vec![(1, 1_000), (2, 1_000), (3, 1_000)]));

		// checks if it can create a pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 300, 500));
		assert_eq!(Assets::total_balance(1, &1), 700);
		assert_eq!(Assets::total_balance(2, &1), 500);

		// Check the balance of liquidity tokens 
		assert_eq!(Assets::total_balance(Dex::create_liquidity_id(2, 1), &1), 387);

		// setup second account
		assert_ok!(Dex::setup_account(2, 1_000_000, vec![(1, 1_000), (2, 1_000), (3, 1_000)]));

		// should not work, because the user doesn't have liquidity tokens
		assert_noop!(Dex::remove_liquidity(RuntimeOrigin::signed(2), 1, 2, 10, 30, 100), Error::<Test>::NotEnoughLiquidityTokens);

		// remove liquidity
		assert_ok!(Dex::remove_liquidity(RuntimeOrigin::signed(1), 1, 2, 10, 30, 100));
		assert_eq!(Assets::total_balance(1, &1), 777);
		assert_eq!(Assets::total_balance(2, &1), 629);

		// Check the balance of liquidity tokens 
		assert_eq!(Assets::total_balance(Dex::create_liquidity_id(2, 1), &1), 287);

	});
}

#[test]
fn create_pool_and_remove_it() {
	new_test_ext().execute_with(|| {
		//create_multiple_accounts();
		//assert_ok!(Dex::create_pool(RuntimeOrigin::root(), 1, 2, 100, 100));
		assert_ok!(Dex::setup_account(1, 1_000_000, vec![(1, 1_000), (2, 1_000), (3, 1_000)]));

		// checks if it can create a pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 300, 500));
		assert_eq!(Assets::total_balance(1, &1), 700);
		assert_eq!(Assets::total_balance(2, &1), 500);

		// Check the balance of liquidity tokens 
		assert_eq!(Assets::total_balance(Dex::create_liquidity_id(2, 1), &1), 387);

		// The native asset is decreased as a result of calling the pool
		assert_eq!(Balance::balance(&1), 999_000);

		// setup second account
		assert_ok!(Dex::setup_account(2, 1_000_000, vec![(1, 1_000), (2, 1_000), (3, 1_000)]));

		// should not work, because the user doesn't have liquidity tokens
		assert_noop!(Dex::remove_pool(RuntimeOrigin::signed(2), 1, 2), Error::<Test>::NotEnoughLiquidityTokens);

		// remove liquidity
		assert_ok!(Dex::remove_pool(RuntimeOrigin::signed(1), 1, 2));
		assert_eq!(Assets::total_balance(1, &1), 1_000);
		assert_eq!(Assets::total_balance(2, &1), 1_000);

		// Check the balance of liquidity tokens 
		assert_eq!(Assets::total_balance(Dex::create_liquidity_id(2, 1), &1), 0);

		// The native asset is decreased as a result of calling the pool
		assert_eq!(Balance::balance(&1), 1_000_000);

	});
}


#[test]
fn create_pool_and_swap() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::setup_account(1, 1_000_000, vec![(1, 100_000), (2, 100_000), (3, 1_000)]));

		// checks if it can create a pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 30_000, 50_000));

		// setup second account
		assert_ok!(Dex::setup_account(2, 1_000_000, vec![(1, 1_000), (2, 1_000), (3, 1_000)]));

		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(2), 1, 2, 300, 450));

		assert_eq!(Assets::total_balance(1, &2), 1000 - 300);
		assert_eq!(Assets::total_balance(2, &2), 1471);
	});
}

#[test]
fn create_pool_and_swap_exact_out() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::setup_account(1, 1_000_000, vec![(1, 1_000_000), (2, 1_000_000), (3, 1_000)]));

		// checks if it can create a pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 300_000, 500_000));

		// setup second account
		assert_ok!(Dex::setup_account(2, 1_000_000, vec![(1, 100_000), (2, 100_000), (3, 1_000)]));

		assert_ok!(Dex::swap_in_for_exact_out(RuntimeOrigin::signed(2), 1, 2, 4_300, 5_000));

		//assert_eq!(Assets::total_balance(1, &2), 100_000 - 3_060);
		//assert_eq!(Assets::total_balance(2, &2), 100_000 + 5_000);
	});
}

#[test]
fn create_pool_and_swap_uniswap_example() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::setup_account(1, 1_000_000, vec![(1, 10_000_000), (2, 10000_000), (3, 1_000)]));

		// checks if it can create a pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 120_0000, 40_0000));

		// setup second account
		assert_ok!(Dex::setup_account(2, 1_000_000, vec![(1, 10_000_000), (2, 10_000_000), (3, 1_000)]));

		assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(2), 1, 2, 3009, 900));

		assert_eq!(Assets::total_balance(1, &2), 10_000_000 - 3009);
		assert_eq!(Assets::total_balance(2, &2), 10000951);

		// transaction should fail
		assert_noop!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(2), 1, 2, 3000, 11_000), Error::<Test>::InsufficientBalance);

	});
}

// Checks if the pool creator makes a profit after a couple of transactions
#[test]
fn check_if_pool_creator_makes_a_profit() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::setup_account(1, 1_000_000, vec![(1, 100_000_000), (2, 100_000_000), (3, 1_000)]));

		// creates a pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 1_000_000, 1_000_000));

		// setup second account
		assert_ok!(Dex::setup_account(2, 1_000_000, vec![(1, 10_000_000), (2, 10_000_000), (3, 1_000)]));

		for _i in 0..=100 {
			assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(2), 1, 2, 100, 95));
			assert_ok!(Dex::swap_exact_in_for_out(RuntimeOrigin::signed(2), 2, 1, 100, 95));
		}
		// the transacting person makes a loss
		assert_eq!(Assets::total_balance(1, &2), 9999596 );
		assert_eq!(Assets::total_balance(2, &2), 9999500);

		// remove pool
		assert_ok!(Dex::remove_pool(RuntimeOrigin::signed(1), 1, 2));
		// the person who creates a pool makes a profit
		assert_eq!(Assets::total_balance(1, &1), 100000404);
		assert_eq!(Assets::total_balance(2, &1), 100_000_000 + 500);

	});
}

// Checks if the pool creator makes a profit after a couple of transactions
#[test]
fn check_if_pool_creator_makes_a_profit_exact_out() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::setup_account(1, 10_000_000, vec![(1, 10_000_000), (2, 10_000_000), (3, 1_000)]));

		// creates a pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 10_000_000, 10_000_000));

		// setup second account
		assert_ok!(Dex::setup_account(2, 1_000_000, vec![(1, 10_000_000), (2, 10_000_000), (3, 1_000)]));

		for _i in 0..=100 {
			assert_ok!(Dex::swap_in_for_exact_out(RuntimeOrigin::signed(2), 1, 2, 200, 100));
			assert_ok!(Dex::swap_in_for_exact_out(RuntimeOrigin::signed(2), 2, 1, 200, 100));
		}
		// the transacting person makes a loss
		assert_eq!(Assets::total_balance(1, &2), 9999495);
		assert_eq!(Assets::total_balance(2, &2), 9999596);

		// remove pool
		assert_ok!(Dex::remove_pool(RuntimeOrigin::signed(1), 1, 2));
		// the person who creates a pool makes a profit
		assert_eq!(Assets::total_balance(1, &1), 10_000_000 + 505);
		assert_eq!(Assets::total_balance(2, &1), 10_000_000 + 404);

	});
}

#[test]
fn test_get_price() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::setup_account(1, 10_000_000, vec![(1, 10_000_000), (2, 10_000_000), (3, 1_000)]));

		// creates a pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 100_000, 200_000));

		assert_eq!(Dex::get_price(1, 2, 200), Ok(400));
	});
}

#[test]
fn test_get_price_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(Dex::get_price(1, 2, 200), Error::<Test>::PoolDoesNotExist);
	});
}

#[test]
fn try_to_create_pool_in_different_orders() {
	new_test_ext().execute_with(|| {
		assert_ok!(Dex::setup_account(1, 10_000_000, vec![(1, 10_000_000), (2, 10_000_000), (3, 1_000)]));
		// creates a pool
		assert_ok!(Dex::create_pool(RuntimeOrigin::signed(1), 1, 2, 100_000, 200_000));
		assert_noop!(Dex::create_pool(RuntimeOrigin::signed(1), 2, 1, 100_000, 200_000), Error::<Test>::PoolAlreadyExists);

	});
}

