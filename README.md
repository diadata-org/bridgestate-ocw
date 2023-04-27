# bridgestate-ocw

## About the Collateral Reader Pallet

The pallet reads the state of various tokens, such as issued tokens, minted tokens, and locked tokens. An offchain worker updates the asset statistics periodically based on the configured time.


The pallet defines the AssetCollector trait, which includes the following methods:

These methods allow you to retrieve information about supported assets, locked and issued amounts of a given asset, the minted asset associated with a given asset, and the associated assets of a minted asset."


```rust
   pub trait AssetCollector {
        fn get_supported_assets(&self) -> Vec<Asset>;
        fn get_locked(asset: Vec<u8>) -> u64;
        fn get_issued(asset: Vec<u8>) -> u64;
        fn get_minted_asset(asset: Vec<u8>) -> Vec<u8>;
        fn get_associated_assets( minted_asset: Vec<u8>) -> Vec<u8>;
    }

```

To use this Pallet on your node, you need to define the required methods and obtain the corresponding values.


## Add the Collateral Reader pallet to your runtime.

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
impl pallet_collateral_reader::Config for Runtime{
    type RuntimeEvent = RuntimeEvent;
    type AssetHelper = crate::pallet_collateral_reader::AssetData;
    type Currency =  Escrow;
    type TokenName = ();
}
```"

```



