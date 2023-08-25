# Bridge Attestation Oracle

DIA’s ‘Bridge Attestation Oracle’ enables the on-chain verification of bridge balances across multiple chains: ”Proof-of-Collateral”. With the oracle’s information, dApps on Polkadot parachains will be able to power security modules to, for instance, trigger automated precautionary notifications and actions when a bridge’s balance drops unexpectedly.

## How it works

The solution consists of 3 key components:

### 1. Off-chain worker

The [Off-chain worker is a Polkadot native feature](https://forum.polkadot.network/t/offchain-workers-design-assumptions-vulnerabilities/2548) that allows validators to aggregate and compute data off-chain in a trustless way, and submit the end result on-chain for protocols to consume. DIA leverages the off-chain worker's functionality to retrieve bridge token balances and issuance. This will help to evaluate the collateralization ratio of each token.

### 2. Community-driven bridge integrations

The second key component of the solution is an open-source library for bridge integrations. As numerous bridges exist in Web3, each with distinct architectures, tracking the token balances of each bridge becomes a complicated task. Therefore, DIA-built bridge aggregators are open-sourced, allowing anyone to write an adaptor for any bridge. We aim to achieve complete decentralization and community-driven development for the entire solution in order to increase trust and scalability.
Currently there are two bridges integrated in the pallet:
- Interlay brigde
- Multichain bridge

### 3. Bridge Attestation Oracle (Collateral Value)

The core feature of the entire solution is providing bridge stakeholders the ability to know if each bridge is fully collateralized at any given time. This solution will be achieved by tracking bridges' locked assets against and issued assets across multiple chains. This enables the calculator of collateral ratios, which protocols can use to define and trigger safety procedures in their code.

- **Example 1: Lending protocol.** A dApp provides a cross-chain token lending market. If a bridge listing the token suddenly becomes undercollateralized, the Bridge Attestation Oracle monitors the state and updates the collateral ratio. The Lending Protocol can use this data to automate actions (e.g. halt operations, liquidate), notify users, and more.
- **Example 2: Monitoring dashboards.** The oracle's real-time data can be used to create dashboards or trigger alerts / notification via social media bots on Twitter, Telegram, or Discord to warn stakeholder communities in case collateral ratios drop below certain thresholds.

In order to use the collateral ratios, Parachains can integrate a Polkadot Pallet (Polkadot native feature), making all the core functionalities of the Bridge Attestation Oracle available for dApps running on the Parachain. Each parachain will be able to decide if they want to make the oracle a public good. The oracle can be customised to provide updated collateral ratios on every block or alternatively, on a request basis, where values will be updated based on dApps’ request.

Following sections explain how to set-up and run the bridge attestation oracle.

## Running the oracle

### About the Collateral Reader Pallet

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

To use this Pallet on your node, you need to define the required methods and obtain the corresponding values. Alternatively, you can use [Substrate Node Template](https://github.com/substrate-developer-hub/substrate-node-template) for testing purposes.

#### Add the Collateral Reader pallet to your runtime

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

#### Running Substrate Node Example with Collateral Reader Pallet

This repository provides an example of a Substrate node configured with a custom pallet - the "Collateral Reader" pallet.  

```sh
git clone git@github.com:diadata-org/bridgestate-ocw.git

```

##### Start the node and dev network by running

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

#### Reading Collateral values

The following section explains how to read collateral values in three different methods. Currently all methods return collateral information for two distinctive bridges:
- Interlay
- Multichain

##### Reading Collateral values using Browser

To access the current state of supported Integration Assets, you can utilize the assetStatsStorage storage of the collateralReader pallet. This will give you insights into the collateral values of various assets integrated into the system.

Please follow the instructions below to retrieve the collateral values:

- Open your web browser and go to https://polkadot.js.org/apps/#/explorer.
- In the developer tab of the Polkadot Apps Explorer, navigate to the `Chain State` section.
- Find and select the `collateralReader` pallet from the dropdown menu in the `Chain State` section.
- Access the assetStatsStorage to view the current state of supported assets, including `Interlay` and `Multichain` assets.



##### Reading Collateral values using `@polkadot/api`

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
