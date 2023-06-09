# bridgestate-ocw

## About the Collateral Reader Pallet

The pallet reads the state of various tokens, such as issued tokens, minted tokens, and locked tokens. An offchain worker updates the asset statistics periodically based on the configured time.

The pallet defines the AssetCollector trait, which includes the following methods:

These methods allow you to retrieve information about supported assets, locked and issued amounts of a given asset, the minted asset associated with a given asset, and the associated assets of a minted asset."

```rust

pub  trait  AssetCollector {

// Returns a list of all assets supported by the chain.
fn  get_supported_assets(&self) -> Vec<Asset>;

//  Returns the amount of the specified asset that is currently locked.
fn  get_locked(asset: Vec<u8>) -> u64;

// Returns the total issued amount of the specified asset.
fn  get_issued(asset: Vec<u8>) -> u64;

// Returns the minted asset associated with the specified asset
fn  get_minted_asset(asset: Vec<u8>) -> Vec<u8>;

// Returns the assets associated with the specified minted asset.( Used by Bridge Adaptor(wip))
fn  get_associated_assets( minted_asset: Vec<u8>) -> Vec<u8>;

}

```

To use this Pallet on your node, you need to define the required methods and obtain the corresponding values.

### Add the Collateral Reader pallet to your runtime

In your Cargo.toml file, include the following line:

```yml

pallet-collateral-reader = { path = "../../crates/collateral-reader", default-features = false }

```

Then, in construct_runtime, add the following line:

```rust

CollateralReader: pallet_collateral_reader::{Pallet, Call, Storage, Event<T>} = 110,

```

Next, implement the collateral reader using the following code:

```rust

impl  pallet_collateral_reader::Config  for  Runtime{

type  RuntimeEvent = RuntimeEvent;

type  AssetHelper = crate::pallet_collateral_reader::AssetData;

type  AuthorityId = crate::pallet_collateral_reader::crypto::TestAuthId;

type  MaxVec = ConstU32<100>;

type  GracePeriod = ConstU32<10>;

}

```"

 - **AuthorityId** : The identifier type for an offchain worker.
 - **AssetHelper** : Helper type for retriving asset stats.
 - **MaxVec** : Specifies the maximum length of a vector that can be for Asset Name
 - **GracePeriod** : Specifies the number of blocks during which offchain worker will update data on chain

```

### Running Substrate Node Example with Collateral Reader Pallet

This repository provides an example of a Substrate node configured with a custom pallet - the "Collateral Reader" pallet.  

```sh
git clone git@github.com:nnn-gif/substrate-node.git
git submodule update --init --recursive

```

#### Start the node and dev network by running

```sh
cargo build --release
cargo run -- --dev
```

Create an account or add a subkey to an existing account, e.g. the example account `Alice` via RPC

```sh
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d \
  '{
 "jsonrpc":"2.0",
 "id":1,
 "method":"author_insertKey",
 "params": [
 "dia!",
 "bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice",
 "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"
 ]
 }'

```

#### Using Docker

Clone substrate-node repo

```sh
git clone git@github.com:nnn-gif/substrate-node.git
git submodule update --init --recursive

```

Build using dockerfile provided

```sh
sh ./docker/build.sh
```
