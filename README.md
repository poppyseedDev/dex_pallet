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

#### TODO
- Create some kind of asset or marketplace where users can use any token to purchase resources, using the price oracle to make sure users pay enough.
- Integrate other DeFi utilities on top of your DEX.

---

## [Substrate Node Template](https://github.com/substrate-developer-hub/substrate-node-template)

A fresh FRAME-based [Substrate](https://www.substrate.io/) node, ready for hacking :rocket:

### Setup

Please first check the latest information on getting starting with Substrate dependencies required to build this project [here](https://docs.substrate.io/main-docs/install/).

### Development Testing

To test while developing, without a full build (thus reduce time to results):

```sh
cargo t -p pallet-dex
cargo t -p <other crates>
```

### Build

Build the node without launching it, with `release` optimizations:

```sh
cargo b -r
```

### Run

Build and launch the node, with `release` optimizations:

```sh
cargo r -r -- --dev
```

### CLI Docs

Once the project has been built, the following command can be used to explore all CLI arguments and subcommands:

```sh
./target/release/node-template -h
```
