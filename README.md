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
fn  get_locked(self, asset: Vec<u8>) -> u64;

// Returns the total issued amount of the specified asset.
fn  get_issued(self, asset: Vec<u8>) -> u64;

// Returns the minted asset associated with the specified asset
fn  get_minted_asset(self,asset: Vec<u8>) -> Vec<u8>;

// Returns the assets associated with the specified minted asset.( Used by Bridge Adaptor(wip))
fn  get_associated_assets( sefl, minted_asset: Vec<u8>) -> Vec<u8>;

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
git clone git@github.com:diadata-org/bridgestate-ocw.git

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

### Reading Collateral values

#### Reading Collateral values using Browser

To access the current state of supported Integration Assets, you can utilize the assetStatsStorage storage of the collateralReader pallet. This will give you insights into the collateral values of various assets integrated into the system.

Please follow the instructions below to retrieve the collateral values:

- Open your web browser and go to https://polkadot.js.org/apps/#/explorer.
- In the developer tab of the Polkadot Apps Explorer, navigate to the `Chain State` section.
- Find and select the `collateralReader` pallet from the dropdown menu in the `Chain State` section.
- Access the assetStatsStorage to view the current state of supported assets, including `Interlay` and `Multichain` assets.



#### Reading Collateral values using `@polkadot/api`

```ts
const { ApiPromise, WsProvider } = require('@polkadot/api');

 const nodeEndpoint = 'ws://127.0.0.1:9944';

async function readChainState() {
   const provider = new WsProvider(nodeEndpoint);
  const api = await ApiPromise.create({ provider });

  try {
     const palletName = 'collateralReader';

     const storageFunction = 'assetStatsStorage';

     const chainState = await api.query[palletName][storageFunction].entries();

    console.log('Chain state:', chainState);
  } catch (error) {
    console.error('Error reading chain state:', error);
  } finally {
     await api.disconnect();
  }
}

readChainState();
```

Clone substrate-node repo

```sh
git clone git@github.com:diadata-org/bridgestate-ocw.git

```

Build using dockerfile provided

```sh
sh ./docker/build.sh
```
