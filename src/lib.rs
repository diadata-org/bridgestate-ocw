#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;
extern crate alloc;

#[frame_support::pallet]
pub mod pallet {
    use codec::{Decode, Encode, EncodeLike};
    use frame_support::{pallet_prelude::*, Parameter, StorageHasher};
    use frame_system::pallet_prelude::*;
    use scale_info::prelude::format;
    use serde_json::Value;
    use sp_runtime::offchain::http;
    use sp_std::{convert::TryInto, str, vec, vec::Vec};

    use frame_support::{
        sp_runtime::offchain::{storage::StorageValueRef, Duration},
        traits::ReservableCurrency,
        Twox128, Twox64Concat,
    };

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    #[pallet::disable_frame_system_supertrait_check]
    pub trait Config: frame_system::Config {
        type Currency: ReservableCurrency<Self::AccountId>;
        type AssetHelper: AssetCollector;
        type TokenName: Parameter + Member + Copy + MaybeSerializeDeserialize + Ord + TypeInfo + MaxEncodedLen;
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    pub trait AssetCollector {
        fn get_supported_assets(&self) -> Vec<Asset>;
        fn get_locked(asset: Vec<u8>) -> u64;
        fn get_issued(asset: Vec<u8>) -> u64;
        fn get_minted_asset(asset: Vec<u8>) -> Vec<u8>;
        fn get_associated_assets( minted_asset: Vec<u8>) -> Vec<u8>;
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
    pub struct Asset {
        address: Vec<u8>,
        metadata: Vec<u8>, //optional
        chain: Vec<u8>,
        decimals: u64,
        symbol: Vec<u8>,
        name: Vec<u8>,
    }

    pub struct AssetData {}
    pub type TokenName = u32;

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
    pub struct BridgeSet {
        locked_assets: Vec<Asset>,
        minted_asset: Asset,
        minted_amount: u64,
    }

    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
    pub struct AssetStats {
        asset: Vec<u8>,
        issued: u64,
        locked: u64,
        minted_asset: Vec<u8>,
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        fn offchain_worker(_: T::BlockNumber) {
           start_ocw()
        }
    }

    fn start_ocw(){
        let ad: AssetData = AssetData {};
        for asset in ad.get_supported_assets() {
            log::info!("assets: {:?}", asset);
            // let storage_double_key = generate_doubee_storage_key(b"Tokens", b"TotalIssuance", b"DOT");

            let data = fetch_data(
                "state_getStorage",
                "0x99971b5749ac43e0235e41b0d378691857c875e4cff74148e4628f264b974c8001a12dfa1fa4ab9a0000",
            );

            log::info!("data: fetched{:?}", &data);

            if let Ok(data) = data {
                let resp: Result<Value, _> = serde_json::from_slice(&data);

                match resp {
                    Ok(response) => log::info!("data: result {:?}", response["result"]),
                    Err(_) => log::info!("err"),
                }
            }
        }

    }
    impl AssetCollector for AssetData {
        fn get_supported_assets(&self) -> Vec<Asset> {
            let mut assets: Vec<Asset> = Vec::new();
            assets.push(Asset {
                address: b"0".to_vec(),
                chain: b"interlay".to_vec(),
                metadata: b"".to_vec(),
                decimals: 0,
                symbol: b"DOT".to_vec(),
                name: b"DOT".to_vec(),
                // storage_key:
                // b"0x99971b5749ac43e0235e41b0d378691857c875e4cff74148e4628f264b974c8001a12dfa1fa4ab9a0000".to_vec()
            });
            assets.push(Asset {
                address: b"1".to_vec(),
                chain: b"interlay".to_vec(),
                metadata: b"".to_vec(),
                decimals: 0,
                symbol: b"INTR".to_vec(),
                name: b"INTR".to_vec(),
                // storage_key:
                // b"0x99971b5749ac43e0235e41b0d378691857c875e4cff74148e4628f264b974c80c483de2de1246ea70002".to_vec()
            });
            assets.push(Asset {
                address: b"2".to_vec(),
                chain: b"interlay".to_vec(),
                metadata: b"".to_vec(),
                decimals: 0,
                symbol: b"IBTC".to_vec(),
                name: b"IBTC".to_vec(),
                // storage_key:
                // b"0x99971b5749ac43e0235e41b0d378691857c875e4cff74148e4628f264b974c80d67c5ba80ba065480001".to_vec()
            });
            assets
        }

        fn get_locked(asset: Vec<u8>) -> u64 {
            45
        }
        fn get_issued(asset: Vec<u8>) -> u64 {
            44
        }
        fn get_minted_asset(asset: Vec<u8>) -> Vec<u8> {
            b"LDOT".to_vec()
        }

        fn get_associated_assets( minted_asset: Vec<u8>) -> Vec<u8> {
            b"DOT".to_vec()
        }
    }

    // fn get_storage_key(metadata: &RuntimeMetadataV14, module_name: &str, storage_item_name: &str)  {
    //      for pallet in metadata.pallets.iter() {
    //         if pallet.name == module_name {
    //              for storage in pallet.storage.as_ref().unwrap().entries.iter() {
    //                      log::info!("storage_item_name: {:?}", storage.name);
    //                      log::info!("storage_item_name: {:?}", storage.docs);
    //                      log::info!("storage_item_name: {:?}", storage.modifier);
    //                      log::info!("storage_item_name: {:?}",  storage.ty);
    //             }
    //         }
    //     }

    //     for types in metadata.types.types.iter() {

    //        if types.id == 50{
    //         log::info!("types: {:?}", types.ty);
    //        }

    //     }

    // }

    // fn generate_storage_key(module_prefix: &[u8], storage_item_prefix: &[u8]) -> Vec<u8> {
    //     let mut storage_key = Vec::new();

    //     let module_hash = Twox128::hash(module_prefix);
    //     let storage_hash = Twox128::hash(storage_item_prefix);

    //     storage_key.extend_from_slice(&module_hash);
    //     storage_key.extend_from_slice(&storage_hash);

    //     storage_key
    // }

    // fn generate_doubee_storage_key(module_prefix: &[u8], storage_item_prefix: &[u8], id: &[u8]) -> Vec<u8> {
    //     let mut storage_key = Vec::new();

    //     let module_hash = Twox128::hash(module_prefix);
    //     let storage_hash = Twox128::hash(storage_item_prefix);
    //     let id_hash = Twox64Concat::hash(id);

    //     storage_key.extend_from_slice(&module_hash);
    //     storage_key.extend_from_slice(&storage_hash);
    //     storage_key.extend_from_slice(&id_hash);

    //     storage_key
    // }

    fn fetch_data(method: &str, param: &str) -> Result<Vec<u8>, http::Error> {
        let rpc_url = "https://interlay.api.onfinality.io/public";

        let rpc_request = format!(
            r#"{{
				"jsonrpc": "2.0",
				"id": "1",
				"method": "{}",
				"params": [{}]
			}}"#,
            method, param
        );

        log::info!("request {}", rpc_request);

        let rbody = rpc_request.into_bytes();

        let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(2_000));
        let request = http::Request::default()
            .method(http::Method::Post)
            .url(rpc_url)
            .body(vec![rbody])
            .add_header("Content-Type", "application/json");

        let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;
        let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
        if response.code != 200 {
            log::info!("Unexpected status code: {}", response.code);
            return Err(http::Error::Unknown);
        }

        let body = response.body().collect::<Vec<u8>>();

        let body_str = sp_runtime::sp_std::str::from_utf8(&body).map_err(|_| {
            log::info!("No UTF8 body");
            http::Error::Unknown
        })?;
        Ok(body)
    }

    // fn read_offchain_storage(key: &[u8]) -> Option<u32> {
    //     let storage_value_ref = StorageValueRef::persistent(key);
    //     storage_value_ref.get::<u32>().unwrap_or(None)
    // }

    // fn save_offchain_storage(key: &[u8], value: &u32) {
    //     let storage_value_ref = StorageValueRef::persistent(key);
    //     storage_value_ref.set(&value);
    // }

    #[pallet::storage]
    #[pallet::unbounded]
    #[pallet::getter(fn get_asset_stats)]
    pub(super) type AssetStatsStorage<T: Config> =
        StorageMap<_, Blake2_128Concat, T::TokenName, AssetStats, OptionQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SomethingStored { something: u32, who: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        NoneValue,
        StorageOverflow,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
        pub fn save_asset_stats(origin: OriginFor<T>, token: T::TokenName) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let locked = T::AssetHelper::get_locked(b"asset".to_vec());
            let issued = T::AssetHelper::get_issued(b"asset".to_vec());
            let minted_asset = T::AssetHelper::get_minted_asset(b"asset".to_vec());
            <AssetStatsStorage<T>>::insert(
                token,
                AssetStats {
                    asset: b"asset".to_vec(),
                    issued,
                    locked,
                    minted_asset,
                },
            );
            Ok(())
        }
    }
}
