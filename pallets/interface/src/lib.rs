#![cfg_attr(not(feature = "std"), no_std)]

// Note that these interfaces should not limit or heavily influence the design of your pallet.
//
// These interfaces do NOT make sense to expose as the extrinsics of your pallet.
// Instead, these will simply be used to execute unit tests to verify the basic logic of your
// pallet is working. You should design your own extrinsic functions which make sense for
// exposing to end users of your pallet.
//
// It should be totally possible to create more complex or unique pallets, while adhering to
// the interfaces below.
//
// If any of these interfaces are not compatible with your design or vision, talk to an
// instructor and we can figure out the best way forward.

use core::{cmp::Ord, fmt::Debug};
use frame_support::{
	dispatch::Vec,
	pallet_prelude::{
		DispatchError, DispatchResult, MaxEncodedLen, MaybeSerializeDeserialize, Member, Parameter,
	},
	traits::tokens::{AssetId as AssetIdTrait, Balance as BalanceTrait},
};

/// A minimal interface to test the functionality of the DEX Pallet.
pub trait DexInterface {
	/// The type which can be used to identify accounts.
	type AccountId: Parameter + Member + MaybeSerializeDeserialize + Debug + Ord + MaxEncodedLen;
	/// The type used to identify various fungible assets.
	type AssetId: AssetIdTrait;
	/// The type used to represent the balance of a fungible asset.
	type AssetBalance: BalanceTrait;

	/// A helper function to setup an account so it can hold any number of assets.
	fn setup_account(who: Self::AccountId) -> DispatchResult;

	/// Do whatever is needed to give user some amount of an asset.
	fn mint_asset(
		who: Self::AccountId,
		token_id: Self::AssetId,
		amount: Self::AssetBalance,
	) -> DispatchResult;

	/// Get a user's asset balance.
	fn asset_balance(who: Self::AccountId, token_id: Self::AssetId) -> Self::AssetBalance;

	/// Return the number of basis points (1/100) used for swap fees.
	fn swap_fee() -> u16;

	// /// Get the LP Token ID that will be generated by creating a pool of `asset_a` and `asset_b`.
	// fn lp_id(asset_a: Self::AssetId, asset_b: Self::AssetId) -> Self::AssetId;

	// /// Add liquidity to a pool on behalf of the user. If needed this will create the pool.
	// ///
	// /// LP tokens are minted to the caller which are used to represent
	// /// "ownership" of the pool.
	// fn add_liquidity(
	// 	who: Self::AccountId,
	// 	asset_a: Self::AssetId,
	// 	asset_b: Self::AssetId,
	// 	amount_a: Self::AssetBalance,
	// 	amount_b: Self::AssetBalance,
	// ) -> DispatchResult;

	/// Removes liquidity from the pool on behalf of the user.
	///
	/// `token_amount` represents the amount of LP tokens to be burned in exchange for underlying
	/// assets.
	fn remove_liquidity_(
		who: Self::AccountId,
		asset_a: Self::AssetId,
		asset_b: Self::AssetId,
		token_amount: Self::AssetBalance,
	) -> DispatchResult;

	/// Swaps an exact amount of `asset_in` for a minimum amount of `asset_out` on behalf of `who`.
	///
	/// The swap fee is deducted from the out amount, so it is left in
	/// the pool for LPs.
	fn swap_exact_in_for_out(
		who: Self::AccountId,
		asset_in: Self::AssetId,
		asset_out: Self::AssetId,
		exact_in: Self::AssetBalance,
		min_out: Self::AssetBalance,
	) -> DispatchResult;

	/// Swaps a max amount of `asset_in` for an exact amount of `asset_out` on behalf of `who`.
	///
	/// The swap fee is added to the in amount, and left in the pool for
	/// the LPs.
	fn swap_in_for_exact_out(
		origin: Self::AccountId,
		asset_in: Self::AssetId,
		asset_out: Self::AssetId,
		max_in: Self::AssetBalance,
		exact_out: Self::AssetBalance,
	) -> DispatchResult;
}
