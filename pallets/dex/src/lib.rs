#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{pallet_prelude::*, traits::fungibles};
/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

// This const indicates a fee percentage, it must be a number between 1 and 100:
//const FEE_PERCENTAGE: u32 = 5;

type AssetIdOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
	<T as frame_system::Config>::AccountId,
>>::AssetId;

type BalanceOf<T> = <<T as Config>::NativeBalance as fungible::Inspect<
	<T as frame_system::Config>::AccountId,
>>::Balance;

type AssetBalanceOf<T> = <<T as Config>::Fungibles as fungibles::Inspect<
	<T as frame_system::Config>::AccountId,
>>::Balance;

use frame_support::traits::fungible;

use sp_runtime::traits::{AccountIdConversion, CheckedAdd, CheckedSub, CheckedDiv, One, Zero};

#[frame_support::pallet]
pub mod pallet {
	use crate::*;
	use frame_support::traits::{fungible, fungibles};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: TypeInfo + frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Type to access the Balances Pallet.
		type NativeBalance: fungible::Inspect<Self::AccountId>
			+ fungible::Mutate<Self::AccountId>
			+ fungible::hold::Inspect<Self::AccountId>
			+ fungible::hold::Mutate<Self::AccountId>
			+ fungible::freeze::Inspect<Self::AccountId>
			+ fungible::freeze::Mutate<Self::AccountId>;

		/// Type to access the Assets Pallet.
		type Fungibles: fungibles::Inspect<Self::AccountId, Balance = BalanceOf<Self>>
			+ fungibles::Mutate<Self::AccountId>
			+ fungibles::Create<Self::AccountId>;

		/// Minimal deposit to create the pool
		type MinPoolDeposit: Get<u32>;

		type FeePercentage: Get<u32>;

		/// Origin for admin-level operations, like creating the pool.
		type CreatePoolOrigin: EnsureOrigin<Self::RuntimeOrigin, Success = Self::AccountId>;

		/// The DEX's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<frame_support::PalletId>;
	}

	/// native token balance
	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
	#[scale_info(skip_type_params(T))]
	pub enum BalanceType<T: Config> {
		NativeBalance,
		AssetBalance(AssetIdOf<T>),
	}      

	// #[derive(Encode, Decode, MaxEncodedLen, TypeInfo)]
	// pub struct LiquidityPool<T: Config> {
	// 	pub liquidity_id: AssetIdOf<T>,
	// 	pub reserve_liq: AssetBalanceOf<T>,
	// 	pub reserve_a: AssetBalanceOf<T>,
	// 	pub reserve_b: AssetBalanceOf<T>,
	// }

	// The pallet's runtime storage items.
	// https://docs.substrate.io/main-docs/build/runtime-storage/
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://docs.substrate.io/main-docs/build/runtime-storage/#declaring-storage-items
	pub type LiquidityPoolMap<T: Config> =
		StorageMap<_, Blake2_128Concat, (AssetIdOf<T>, AssetIdOf<T>), T::AccountId>;

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/main-docs/build/events-errors/
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A liquidity pool has been created.
		PoolCreated { asset_a: AssetIdOf<T>, asset_b: AssetIdOf<T>, liquidity_id: AssetIdOf<T> },
		LiquidityAdded { asset_a: AssetIdOf<T>, asset_b: AssetIdOf<T>, liquidity_id: AssetIdOf<T> },
		LiquidityRemoved { asset_a: AssetIdOf<T>, asset_b: AssetIdOf<T>, liquidity_id: AssetIdOf<T>, amount_liq: AssetBalanceOf<T>},
		PoolRemoved { asset_a: AssetIdOf<T>, asset_b: AssetIdOf<T>, liquidity_id: AssetIdOf<T> },
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		/// The user has insufficient balance
		InsufficientBalance,
		/// Cannot create new pool, since pool already exists!
		PoolAlreadyExists,
		/// Cannot add liquidity to the pool, if the pool does not exist!
		PoolDoesNotExist,
		/// This pool is empty and doesn't have any assets in the pool
		EmptyPool,
		/// Not enough asset A
		NotEnoughAssetsA,
		/// Not enough asset B
		NotEnoughAssetsB,
		/// Not enough liquidity tokens
		NotEnoughLiquidityTokens,
		/// Requested asset amount for widthdrawl exceeds the amount asset currently in the pool
		RequestedExceedsPoolBalance,

	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {

		/// CreatePool: This function allows users to create a new liquidity pool for a token pair.
		/// It takes as input the two tokens and the amount of each token the user wants to deposit.
		/// This will mint LP (Liquidity Provider) tokens and transfer them to the user's address.
		/// 
		/// This function fails if the pool has already been created
		/// Registers new pool for a given asset pair a, b in the asset registry
  		/// Asset registry creates new id or returns previously created one if such pool existed before.
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn create_pool(
			origin: OriginFor<T>, 
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			amount_a: AssetBalanceOf<T>,
			amount_b: AssetBalanceOf<T>,
		) -> DispatchResult {
			
			// Check origin
			let who = T::CreatePoolOrigin::ensure_origin(origin)?;
			
			// order assets_a and assets_b
			let (asset_a, asset_b, amount_a, amount_b) = Self::order_asset_ids(asset_a, asset_b, amount_a, amount_b)?;

			// Ensure the caller has enough balances in both of the assets, where he is providing liquidity
			// and the that the balances of both assets are greater then minimum value required
			Self::check_user_balances(&who, &asset_a, &asset_b, &amount_a, &amount_a)?;

			// Create liquidity_id from asset_a and asset_b
			let lp_asset_id: AssetIdOf<T> = Self::create_liquidity_id(asset_a.clone(), asset_b.clone());
			
			// Liquidity amount is equal to sqrt(amount_a * amount_b)
			let amount_liq: AssetBalanceOf<T> = Self::get_sqrt_of_asset_balance(amount_a.checked_mul(&amount_b).ok_or(ArithmeticError::Overflow)?);
			
			// Check if the liquidity pool already exists
			ensure!(!LiquidityPoolMap::<T>::contains_key(&(asset_a.clone(), asset_b.clone())), Error::<T>::PoolAlreadyExists);

			// Get the pool account
			let pool_account = Self::generate_account_from_asset_id_pair(asset_a.clone(), asset_b.clone());
		
			// native currency is transferd while creating this account
			T::NativeBalance::transfer(&who, &pool_account, BalanceOf::<T>::from(1_000 as u32), Preservation::Expendable)?;
			
			// Save the new liquidity pool in the storage
			LiquidityPoolMap::<T>::insert(&(asset_a.clone(), asset_b.clone()), pool_account.clone());

			// create an event
			Self::deposit_event(Event::<T>::PoolCreated {
				asset_a: asset_a.clone(),
				asset_b: asset_b.clone(),
				liquidity_id: lp_asset_id.clone(),
			});

			// transfer the tokens from the users accout into pool account 
			Self::transfer_assets_a_and_b(&who, &pool_account, &asset_a, &asset_b, &amount_a, &amount_b)?;

			// check if the liquidity token already exists and if not create it
			if !T::Fungibles::asset_exists(lp_asset_id.clone()) {
				let pallet_account = Self::account_id();
				T::Fungibles::create(lp_asset_id.clone(), pallet_account, false, One::one())?;
			}
			// mint the lp tokens into the users account
			T::Fungibles::mint_into(lp_asset_id, &who, amount_liq.clone())?;

			Ok(())
		}


		/// This function allow users to deposit tokens into an the existing liquidity pool.
		/// It should calculate the appropriate amount of LP tokens to mint based on the
		/// existing pool balances and the deposited amount, then transfer the LP tokens to the user's address.
		/// 
		/// This function fails if the pool doesn't exist or if the assets that are present in the account is equal to zero
		/// First ensures that the user has enough assets to deposit their them and
		#[pallet::call_index(2)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn add_liquidity(
			origin: OriginFor<T>,
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			amount_a: AssetBalanceOf<T>,
			amount_b: AssetBalanceOf<T>,
		) -> DispatchResult {
			// Check origin
			let who = T::CreatePoolOrigin::ensure_origin(origin)?;

			// Order the asset ids
			let (asset_a, asset_b, amount_a, amount_b) = Self::order_asset_ids(asset_a, asset_b, amount_a, amount_b)?;
			
			// Ensure the caller has enough balances in both of the assets, where he is providing liquidity
			// and the that the balances of both assets are greater then minimum value required
			Self::check_user_balances(&who, &asset_a, &asset_b, &amount_a, &amount_a)?;

			// Create liquidity_id from asset_a and asset_b
			let lp_asset_id: AssetIdOf<T> = Self::create_liquidity_id(asset_a.clone(), asset_b.clone());
			
			// Check if the liquidity pool already exists
			ensure!(LiquidityPoolMap::<T>::contains_key(&(asset_a.clone(), asset_b.clone())), Error::<T>::PoolDoesNotExist);

			// Get the pool account
			let pool_account = Self::generate_account_from_asset_id_pair(asset_a.clone(), asset_b.clone());
		
			// Get the current reserves
			let reserve_a = T::Fungibles::balance(asset_a.clone(), &pool_account);
			let reserve_b = T::Fungibles::balance(asset_b.clone(), &pool_account);
		
			// Calculate liquidity tokens to mint
			// LP tokens represent the amount of shares a LP provider has deposited.

			// check that the pool is not empty
			ensure!(!reserve_a.is_zero() || !reserve_b.is_zero(), Error::<T>::EmptyPool);

			// get the total issuance of LP tokens 
			let amount_a_in_reserves = amount_a.checked_mul(&T::Fungibles::total_issuance(lp_asset_id.clone())).ok_or(ArithmeticError::Overflow)?;
			let lp_tokens_a = amount_a_in_reserves / reserve_a;
	
			let amount_b_in_reserves = amount_b.checked_mul(&T::Fungibles::total_issuance(lp_asset_id.clone())).ok_or(ArithmeticError::Overflow)?;
			let lp_tokens_b = amount_b_in_reserves / reserve_b;
	
			// Use the smaller one to maintain the ratio
			let lp_tokens = lp_tokens_a.min(lp_tokens_b);

			// create an event
			Self::deposit_event(Event::<T>::LiquidityAdded {
				asset_a: asset_a.clone(),
				asset_b: asset_b.clone(),
				liquidity_id: lp_asset_id.clone(),
			});

			// transfer the tokens from the users accout into pool account 
			Self::transfer_assets_a_and_b(&who, &pool_account, &asset_a, &asset_b, &amount_a, &amount_b)?;

			// Mint the liquidity tokens
			T::Fungibles::mint_into(lp_asset_id, &who, lp_tokens)?;
		
			Ok(())
		}

		/// This function allows users to remove liquidity from an existing liquidity pool. 
		/// Users specify the minimum amount of asset_a and asset_b they wish to receive in return for their liquidity tokens. 
		/// The function first checks whether the pool exists and that the user has enough liquidity tokens.
		/// It then calculates how much of asset_a and asset_b the user should receive based on the ratio of their liquidity tokens to the total supply.
		/// This operation can fail if the user doesn't have enough liquidity tokens, the pool does not exist, or the removal of liquidity would result in the user receiving less than the minimum amount of asset_a or asset_b they specified. 
		/// If successful, the assets are transferred from the pool to the user's account, and the liquidity tokens are burned from the user's account.
		/// 
		/// # Arguments
		/// 
		/// * `origin` - The origin caller of the function, who will be withdrawing liquidity.
		/// * `asset_a` - One of the assets in the liquidity pool.
		/// * `asset_b` - The other asset in the liquidity pool.
		/// * `min_amount_a` - The minimum amount of asset_a that the user expects to receive.
		/// * `min_amount_b` - The minimum amount of asset_b that the user expects to receive.
		/// * `liquidity` - The amount of liquidity tokens the user wishes to redeem.
		#[pallet::call_index(3)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn remove_liquidity(
			origin: OriginFor<T>,
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
			min_amount_a: AssetBalanceOf<T>,
			min_amount_b: AssetBalanceOf<T>,
			amount_liq: AssetBalanceOf<T>,
		) -> DispatchResult {
			// Check origin
			let who = T::CreatePoolOrigin::ensure_origin(origin)?;

			//  Order the asset ids, orders the amounts as well
			let (asset_a, asset_b, min_amount_a, min_amount_b) = Self::order_asset_ids(asset_a, asset_b, min_amount_a, min_amount_b)?;

			// Create liquidity_id from asset_a and asset_b
			let lp_asset_id: AssetIdOf<T> = Self::create_liquidity_id(asset_a.clone(), asset_b.clone());

			// Checks if the user has enough liquidity tokens
			ensure!(T::Fungibles::balance(lp_asset_id.clone(), &who) >= amount_liq, Error::<T>::NotEnoughLiquidityTokens);
			ensure!(!amount_liq.is_zero(), "Balance of liquidity tokens has to be greater then 0");

			// Check if the liquidity pool exists
			ensure!(LiquidityPoolMap::<T>::contains_key(&(asset_a.clone(), asset_b.clone())), Error::<T>::PoolDoesNotExist);

			// Get the pool account
			let pool_account = Self::generate_account_from_asset_id_pair(asset_a.clone(), asset_b.clone());

			// Get the current reserves
			let reserve_a = T::Fungibles::balance(asset_a.clone(), &pool_account);
			let reserve_b = T::Fungibles::balance(asset_b.clone(), &pool_account);

			// Calculate the liquidity amount to remove
			let total_liquidity = T::Fungibles::total_issuance(lp_asset_id.clone());
			
			// reserve_a * amount_liq / total_liquidity

			let res_amount_a = reserve_a.checked_mul(&amount_liq).ok_or(ArithmeticError::Overflow)?;
			let res_amount_b = reserve_b.checked_mul(&amount_liq).ok_or(ArithmeticError::Overflow)?;

			let remove_amount_a = res_amount_a.checked_div(&total_liquidity).ok_or(ArithmeticError::Underflow)?;
			let remove_amount_b = res_amount_b.checked_div(&total_liquidity).ok_or(ArithmeticError::Underflow)?;

			// Ensure the user receives at least the minimum amounts they expect
			ensure!(remove_amount_a >= min_amount_a, Error::<T>::NotEnoughAssetsA);
			ensure!(remove_amount_b >= min_amount_b, Error::<T>::NotEnoughAssetsB);

			// Transfer the tokens from the pool account to the user's account
			Self::transfer_assets_a_and_b(&pool_account, &who, &asset_a, &asset_b, &remove_amount_a, &remove_amount_b)?;

			// Burn the liquidity tokens from the user's account
			T::Fungibles::burn_from(lp_asset_id.clone(), &who, amount_liq, Precision::BestEffort, Fortitude::Polite)?;

			// Emit an event
			Self::deposit_event(Event::<T>::LiquidityRemoved {
				asset_a: asset_a.clone(),
				asset_b: asset_b.clone(),
				liquidity_id: lp_asset_id,
				amount_liq,
			});

			Ok(())
		}


		/// This function allows users to remove the pool and all the liquidity from an existing liquidity pool. 
		/// Users don't have to specify the minimum amount of asset_a or asset_b or any liquidity tokens,
		/// since all the balance that is in the pool is transfered into the callers account
		/// The function first checks whether the pool exists and that the user has enough liquidity tokens.
		/// 
		/// The pool is removed from storage, the native asset that was locked in the pool is transfered to the
		/// users account.
		/// 
		/// # Arguments
		/// 
		/// * `origin` - The origin caller of the function, who will be withdrawing liquidity.
		/// * `asset_a` - One of the assets in the liquidity pool.
		/// * `asset_b` - The other asset in the liquidity pool.
		#[pallet::call_index(4)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn remove_pool(
			origin: OriginFor<T>,
			asset_a: AssetIdOf<T>,
			asset_b: AssetIdOf<T>,
		) -> DispatchResult {
			// Check origin
			let who = T::CreatePoolOrigin::ensure_origin(origin)?;

			//  Order the asset ids, orders the amounts as well
			let (asset_a, asset_b, _, _) = Self::order_asset_ids(asset_a, asset_b, AssetBalanceOf::<T>::zero(), AssetBalanceOf::<T>::zero())?;

			// Create liquidity_id from asset_a and asset_b
			let lp_asset_id: AssetIdOf<T> = Self::create_liquidity_id(asset_a.clone(), asset_b.clone());

			// Check if the liquidity pool exists
			ensure!(LiquidityPoolMap::<T>::contains_key(&(asset_a.clone(), asset_b.clone())), Error::<T>::PoolDoesNotExist);
			
			// Get the pool account
			let pool_account = Self::generate_account_from_asset_id_pair(asset_a.clone(), asset_b.clone());
			
			// Get the current reserves
			let reserve_a = T::Fungibles::balance(asset_a.clone(), &pool_account);
			let reserve_b = T::Fungibles::balance(asset_b.clone(), &pool_account);
			
			// Calculate the liquidity amount to remove
			let total_liquidity = T::Fungibles::total_issuance(lp_asset_id.clone());
			
			// Checks if the user has enough liquidity tokens
			ensure!(T::Fungibles::balance(lp_asset_id.clone(), &who) >= total_liquidity, Error::<T>::NotEnoughLiquidityTokens);

			// Remove the total liquidity
			Self::transfer_assets_a_and_b(&pool_account, &who, &asset_a, &asset_b, &reserve_a, &reserve_b)?;

			// TODO: remove asset from storage

			// Burn the liquidity tokens from the user's account
			T::Fungibles::burn_from(lp_asset_id.clone(), &who, total_liquidity, Precision::BestEffort, Fortitude::Polite)?;

			// Return native tokens to the account issuer:
			T::NativeBalance::transfer(&pool_account, &who, BalanceOf::<T>::from(1_000 as u32), Preservation::Expendable)?;

			// Emit an event
			Self::deposit_event(Event::<T>::PoolRemoved {
				asset_a: asset_a.clone(),
				asset_b: asset_b.clone(),
				liquidity_id: lp_asset_id,
			});

			Ok(())
		}


		/// SwapExactInForOut: This function allows users to swap one token for another.
		/// It takes as input the account of the user, the two tokens to be swapped, the exact amount 
		/// of token to be swapped (in), and the minimum amount of token to be received (out).
		/// 
		/// We take a flat 1% fee from exact_in amount for processing the transaction.
		#[pallet::call_index(5)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn swap_exact_in_for_out(
			origin: OriginFor<T>,
			asset_in: AssetIdOf<T>,
			asset_out: AssetIdOf<T>,
			exact_in: AssetBalanceOf<T>,
			min_out: AssetBalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			//  Order the asset ids, orders the amounts as well
			// Using asset_a and asset_b naming when order matters, but it doesn't matter which one is which
			let (asset_a, asset_b, _, _) = Self::order_asset_ids(asset_in.clone(), asset_out.clone(), AssetBalanceOf::<T>::zero(), AssetBalanceOf::<T>::zero())?;

			// check the pool exists
			ensure!(LiquidityPoolMap::<T>::contains_key(&(asset_a.clone(), asset_b.clone())), Error::<T>::PoolDoesNotExist);
			
			// Get the pool account
			let pool_account = Self::generate_account_from_asset_id_pair(asset_a, asset_b);
			
			// check user has enough balance
			ensure!(T::Fungibles::balance(asset_in.clone(), &who) >= exact_in, Error::<T>::InsufficientBalance);

			// take a flat fee of 1% for processing the transaction
			let percent_remaining = Percent::from_rational(100u32 - T::FeePercentage::get(), 100u32); // (100 - FEE_PERCENTAGE)%
			let reduced_for_fee_in = percent_remaining * exact_in;

			// get pool balances
			let pool_balance_in = T::Fungibles::balance(asset_in.clone(), &pool_account);
			let pool_balance_out = T::Fungibles::balance(asset_out.clone(), &pool_account);

			// calculate amount out
			// formula: amount_out = pool_balance_out - (pool_balance_in * pool_balance_out) / (pool_balance_in + exact_in)
			let part_1 = pool_balance_in.checked_mul(&pool_balance_out).ok_or(ArithmeticError::Overflow)?;
			let part_2 = pool_balance_in.checked_add(&reduced_for_fee_in).ok_or(ArithmeticError::Overflow)?;
			let part_3 = part_1.checked_div(&part_2).ok_or(ArithmeticError::Underflow)?;
			let amount_out = pool_balance_out.checked_sub(&part_3).ok_or(ArithmeticError::Underflow)?;

			// check minimum output
			ensure!(amount_out >= min_out, Error::<T>::InsufficientBalance);

			// update pool balances and user balances
			// transfer from balance into pool
			T::Fungibles::transfer(asset_in, &who, &pool_account, exact_in, Preservation::Expendable)?;
			// transfer from pool to balance
			T::Fungibles::transfer(asset_out,  &pool_account, &who, amount_out, Preservation::Expendable)?;

			Ok(())
		}

		/// SwapInForExactOut: This function allows users to swap one token for another.
		/// It takes as input the account of the user, the two tokens to be swapped, the maximum amount 
		/// of token to be swapped (in), and the exact amount of token to be received (out).
		/// 
		/// We take a flat 1% fee from max_in amount for processing the transaction.
		#[pallet::call_index(6)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn swap_in_for_exact_out(
			origin: OriginFor<T>,
			asset_in: AssetIdOf<T>,
			asset_out: AssetIdOf<T>,
			max_in: AssetBalanceOf<T>,
			exact_out: AssetBalanceOf<T>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;

			// Order the asset ids, orders the amounts as well
			let (asset_a, asset_b, _, _) = Self::order_asset_ids(asset_in.clone(), asset_out.clone(), AssetBalanceOf::<T>::zero(), AssetBalanceOf::<T>::zero())?;
		
			// Check the pool exists
			ensure!(LiquidityPoolMap::<T>::contains_key(&(asset_a.clone(), asset_b.clone())), Error::<T>::PoolDoesNotExist);
		
			// Get the pool account
			let pool_account = Self::generate_account_from_asset_id_pair(asset_a, asset_b);
		
			// Check user has enough balance
			ensure!(T::Fungibles::balance(asset_in.clone(), &who) >= max_in, Error::<T>::InsufficientBalance);
		
			// Get pool balances
			let pool_balance_in = T::Fungibles::balance(asset_in.clone(), &pool_account);
			let pool_balance_out = T::Fungibles::balance(asset_out.clone(), &pool_account);

			// Check if the max_in or exact_out exceed pool balances
			ensure!(pool_balance_out > exact_out, Error::<T>::RequestedExceedsPoolBalance);
		
			// Calculate amount in
			// formula: amount_in = (pool_balance_in * pool_balance_out) / (pool_balance_out - exact_out) - pool_balance_in
			let part_1 = pool_balance_in.checked_mul(&pool_balance_out).ok_or(ArithmeticError::Overflow)?;
			let part_2 = pool_balance_out.checked_sub(&exact_out).ok_or(ArithmeticError::Underflow)?;
			let part_3 = part_1.checked_div(&part_2).ok_or(ArithmeticError::Underflow)?;
			let amount_in_before_fee = part_3.checked_sub(&pool_balance_in).ok_or(ArithmeticError::Underflow)?;
		
			// Take a flat fee of 1% for processing the transaction
			let percent_fee = Percent::from_rational(T::FeePercentage::get(), 100u32); // FEE_PERCENTAGE%
			//let fee = percent_fee * amount_in_before_fee;
			let fee = percent_fee * amount_in_before_fee;
			let amount_in = amount_in_before_fee.checked_add(&fee).ok_or(ArithmeticError::Overflow)?;
		
			// Add a max in amount 
			ensure!(amount_in <= max_in, Error::<T>::InsufficientBalance);
		
			// Update pool balances and user balances
			// Transfer from balance into pool
			T::Fungibles::transfer(asset_in, &who, &pool_account, amount_in, Preservation::Expendable)?;
			// Transfer from pool to balance
			T::Fungibles::transfer(asset_out, &pool_account, &who, exact_out, Preservation::Expendable)?;
		
			Ok(())
		}
	}
}

use frame_support::sp_runtime::traits::Hash;
use sp_runtime::{
	traits::{CheckedMul, TrailingZeroInput},
	ArithmeticError, Percent,
};

use frame_support::traits::{
	fungible::Mutate,
	fungibles::{Create, Inspect, Mutate as FSMutate},
	tokens::{Precision, Fortitude, Preservation}
};

use frame_support::dispatch::Vec;

use sp_arithmetic::traits::IntegerSquareRoot;

impl<T: Config> Pallet<T> {

	/// Orders asset Ids
	/// lp(a, b) == lp(b, a)
	/// idea is that the user can submit assets in any order
	/// 2. we "sort" the assets 3. then create the id
	/// you need to figure out what you want to return if anything
	pub fn order_asset_ids(
		asset_a: AssetIdOf<T>,
		asset_b: AssetIdOf<T>,
		amount_a: AssetBalanceOf<T>,
		amount_b: AssetBalanceOf<T>
	) -> Result<(AssetIdOf<T>, AssetIdOf<T>, AssetBalanceOf<T>, AssetBalanceOf<T>), DispatchError> {
		ensure!(asset_a != asset_b, "cant use the same id twice");
		return if asset_a.encode() > asset_b.encode() {
			Ok((asset_a, asset_b, amount_a, amount_b))
		} else {
			Ok((asset_b, asset_a, amount_b, amount_a))
		}
	}

	// This function is used to ensure that the user has enough balance of assets 
	/// and checks if the balance is greater than the minimum required.
	fn check_user_balances(
		who: &T::AccountId, 
		asset_a: &AssetIdOf<T>, 
		asset_b: &AssetIdOf<T>,
		amount_a: &AssetBalanceOf<T>,
		amount_b: &AssetBalanceOf<T>,
	) -> DispatchResult {
		ensure!(T::Fungibles::balance(asset_a.clone(), who) >= *amount_a, "Not enough balance for asset_a");
		ensure!(T::Fungibles::balance(asset_b.clone(), who) >= *amount_b, "Not enough balance for asset_b");
		ensure!(!amount_a.is_zero(), "Balance of asset_a has to be greater then the min value");
		ensure!(!amount_b.is_zero(), "Balance of asset_b has to be greater then the min value");
		Ok(())
	}

	// Transfer both assets A and B from one account (from) to another account (dest)
	// This function is used to transfer the assets from the user's account to the pool account.
	// Or from user's account to the pool account
	fn transfer_assets_a_and_b(
		from: &T::AccountId, 
		dest: &T::AccountId,
		asset_a: &AssetIdOf<T>, 
		asset_b: &AssetIdOf<T>,
		amount_a: &AssetBalanceOf<T>,
		amount_b: &AssetBalanceOf<T>,
	) -> DispatchResult {
		// It Preservation is expendable so it can transfer the full amount from one account, 
		// even if the amount is less than zero
		T::Fungibles::transfer(asset_a.clone(), from, dest, *amount_a, Preservation::Expendable)?;
		T::Fungibles::transfer(asset_b.clone(), from, dest, *amount_b, Preservation::Expendable)?;
		Ok(())
	}

	// A function to generate unique liquidity_id from asset_a and asset_b
	fn create_liquidity_id(asset_a: AssetIdOf<T>, asset_b: AssetIdOf<T>) -> AssetIdOf<T> {
		// Generate a unique id based on asset_a and asset_b
		// This is a very simplistic approach and might need to be enhanced based on your needs.
		let bytes = T::Hashing::hash(&(asset_a, asset_b).encode());
		let liq_id = AssetIdOf::<T>::decode(&mut TrailingZeroInput::new(&bytes.encode()))
			.expect("in our PBA exam, we assume all bytes can be turned into some account id");
		liq_id
	}

	// This function assumes asset_a and asset_b have already been sorted
	fn generate_account_from_asset_id_pair(
		asset_a: AssetIdOf<T>,
		asset_b: AssetIdOf<T>,
	) -> T::AccountId {
		let bytes = T::Hashing::hash(&(asset_a, asset_b).encode());
		let generated_account = T::AccountId::decode(&mut TrailingZeroInput::new(&bytes.encode()))
			.expect("in our PBA exam, we assume all bytes can be turned into some account id");
		generated_account
	}

	/// The account ID of the dex pallet. It can be used as an admin for new assets created.
	///
	/// This actually does computation. If you need to keep using it, then make sure you cache the
	/// value and only call this once.
	fn account_id() -> T::AccountId {
		T::PalletId::get().into_account_truncating()
	}

	/// Functionality that can be used as a price oracle, enables users to convert the price of the
	/// from one asset to another.
	pub fn get_price (
		asset_in: AssetIdOf<T>,
		asset_out: AssetIdOf<T>,
		amount_in: AssetBalanceOf<T>,
	) -> Result<AssetBalanceOf<T>, DispatchError> {

		//  Order the asset ids, orders the amounts as well
		let (asset_a, asset_b, _, _) = Self::order_asset_ids(asset_in.clone(), asset_out.clone(), AssetBalanceOf::<T>::zero(), AssetBalanceOf::<T>::zero())?;

		// check if the pool exists
		ensure!(LiquidityPoolMap::<T>::contains_key(&(asset_a.clone(), asset_b.clone())), Error::<T>::PoolDoesNotExist);
		
		// Get the pool account
		let pool_account = Self::generate_account_from_asset_id_pair(asset_a, asset_b);
		
		// get pool balances
		let pool_balance_in = T::Fungibles::balance(asset_in.clone(), &pool_account);
		let pool_balance_out = T::Fungibles::balance(asset_out.clone(), &pool_account);

		let part1 = amount_in.checked_mul(&pool_balance_out).ok_or(ArithmeticError::Overflow)?;
		let amount_out = part1.checked_div(&pool_balance_in).ok_or(ArithmeticError::Overflow)?;

		Ok(amount_out)
	}

	/// this function allows you to set up an account where you can provide the native balance amount and
	/// a vector of assets and their balances. This function will create the assets if they dont exist.
	pub fn setup_account(
		who: T::AccountId,
		native_balance: BalanceOf<T>,
		assets: Vec<(AssetIdOf<T>, AssetBalanceOf<T>)>,
	) -> DispatchResult {
		T::NativeBalance::mint_into(&who, native_balance)?;

		// iterate over the assets and mint them into the account
		for (asset_id, asset_balance) in assets {
			// create the asset if it doesnt exist
			if !T::Fungibles::asset_exists(asset_id.clone()) {
				let pallet_account = Self::account_id();
				// is_sufficient is set to true because we don't want users to have an existential deposit to hold this asset
				T::Fungibles::create(asset_id.clone(), pallet_account, true, One::one())?;
			}

			T::Fungibles::mint_into(asset_id.clone(), &who, asset_balance)?;
		}

		Ok(())
	}

	pub fn get_sqrt_of_asset_balance(balance: AssetBalanceOf<T>) -> AssetBalanceOf<T> {
		IntegerSquareRoot::integer_sqrt(&balance)
	}
}

// Look at `../interface/` to better understand this API.
impl<T: Config> pba_interface::DexInterface for Pallet<T> {
	type AccountId = T::AccountId;
	type AssetId = <T::Fungibles as fungibles::Inspect<Self::AccountId>>::AssetId;
	type AssetBalance = <T::Fungibles as fungibles::Inspect<Self::AccountId>>::Balance;

	fn setup_account(_who: Self::AccountId) -> DispatchResult {
		unimplemented!()
	}

	fn mint_asset(
		_who: Self::AccountId,
		_token_id: Self::AssetId,
		_amount: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn asset_balance(_who: Self::AccountId, _token_id: Self::AssetId) -> Self::AssetBalance {
		unimplemented!()
	}

	fn swap_fee() -> u16 {
		T::FeePercentage::get() as u16
	}
	
	/// Allows users to withdraw their tokens from a liquidity pool. It take as input 
	/// amount of asset_a and amount of asset_b that the user wants to recieve
	/// The function looks if the user has enough of liquidity tokens to remove the wanted amount from 
	/// the balance if yes, this amount will be then removed
	fn remove_liquidity_(
		_who: Self::AccountId,
		_asset_a: Self::AssetId,
		_asset_b: Self::AssetId,
		_token_amount: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn swap_exact_in_for_out(
		_who: Self::AccountId,
		_asset_in: Self::AssetId,
		_asset_out: Self::AssetId,
		_exact_in: Self::AssetBalance,
		_min_out: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}

	fn swap_in_for_exact_out(
		_origin: Self::AccountId,
		_asset_in: Self::AssetId,
		_asset_out: Self::AssetId,
		_max_in: Self::AssetBalance,
		_exact_out: Self::AssetBalance,
	) -> DispatchResult {
		unimplemented!()
	}
}