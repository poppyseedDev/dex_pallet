# Decentralized Exchange Pallet

- Uses a multi-asset pallet to represent many tokens.
  - You may create your own or use the included `pallet_assets`.
- A Uniswap Version 2 style DEX to allow users to trustlessly exchange tokens with one another.
  - Implement rewards to incentivize users to create pools.
  - Exposes an API from the pallet which acts as a “price oracle” based on the existing liquidity pools.

### TODO

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
