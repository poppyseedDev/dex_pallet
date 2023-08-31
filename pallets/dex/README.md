## DEX Implementation Overview

This repository implements a decentralized exchange (DEX) inspired by the mechanics of Uniswap V2. The DEX is created on a multi-asset platform, and it leverages the benefits of the included `pallet_assets`, though you are free to implement your own multi-asset pallet. 

Users can trustlessly exchange tokens, incentivized by the fee rewards system which encourages them to create liquidity pools. The implemented API further acts as a "price oracle," drawing data from these existing liquidity pools.

### Uniswap Version 2 Mechanics 

*To be elaborated further*

#### Liquidity Pools

Users can create a liquidity pool by depositing two assets, setting the ratio (K), which is preserved during the swap operations. To regulate the creation of pools, a deposit of the native token is required from the user.

Key functions for managing liquidity pools include:
 - `create_pool`
 - `add_liquidity`
 - `remove_liquidity`
 - `remove_pool`

#### Token Swapping

The DEX ensures the constant product (K) remains constant during swaps. If asset A and asset B are swapped, where `A` is `BALANCE_IN_POOL_OF_ASSET_A` and `B` is `BALANCE_IN_POOL_OF_ASSET_B`, the preservation of K is as follows:

```
A * B = k
(A + a) * (B - b) = k
```

The liquidity providers are rewarded with a 5% flat fee drawn from the depositing asset during swaps. The fee is added directly to the pool balance.

Functions handling swapping include:
 - `swap_exact_in_for_out`
 - `swap_in_for_exact_out`

For detailed information, please refer to the official Uniswap documentation.

#### Future Work: 

Immediate changes to consider include:

##### In `lib.rs`
 - Remove the `StorageMap` from the storage.
 - Make the fee percentage modifiable by moving the `const FEE_PERCENTAGE: u32 = 5;` to the config.
 
##### In `tests.rs`
 - Implement tests for various edge cases.
 - Validate event submissions.

### Run the Code

#### Test
To run tests, use the following command:
```
cargo t -p pallet-dpos
```

#### Build and Run

Build the node with `release` optimizations and launch:

```sh
cargo r -r -- --dev
```

#### CLI Docs

After building the project, you can explore all CLI arguments and subcommands with:

```sh
./target/release/node-template -h
```
