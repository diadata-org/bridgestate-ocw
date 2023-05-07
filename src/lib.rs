#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
use frame_support::{sp_runtime::KeyTypeId, traits::Get};
use frame_system::offchain::{CreateSignedTransaction, SendSignedTransaction};
pub use pallet::*;
use sp_runtime::{
	offchain::storage::{MutateStorageError, StorageRetrievalError, StorageValueRef},
	DispatchError, SaturatedConversion,
};

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"dia!");

pub mod crypto {

	use super::KEY_TYPE;
	use frame_support::sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		MultiSignature, MultiSigner,
	};
	use scale_info::prelude::format;

	app_crypto!(sr25519, KEY_TYPE);

	pub struct TestAuthId;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for TestAuthId {
		type RuntimeAppPublic = Public;
		type GenericSignature = sr25519::Signature;
		type GenericPublic = sr25519::Public;
	}
}

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use codec::{Decode, Encode};
	use frame_support::{pallet_prelude::*};
	use frame_system::{
		offchain::{AppCrypto, Signer},
		pallet_prelude::*,
	};

 	use frame_support::storage::bounded_vec::BoundedVec;
	use sp_std::{convert::TryInto, str, vec, vec::Vec};

	#[pallet::pallet]
	#[pallet::without_storage_info]
	pub struct Pallet<T>(_);

	#[pallet::config]
	#[pallet::disable_frame_system_supertrait_check]
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		type AssetHelper: AssetCollector;
		#[pallet::constant]
		type MaxVec: Get<u32>;

		#[pallet::constant]
		type GracePeriod: Get<Self::BlockNumber>;

		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	pub trait AssetCollector {
		fn get_supported_assets(&self) -> Vec<Asset>;
		fn get_locked(asset: Vec<u8>) -> u64;
		fn get_issued(asset: Vec<u8>) -> u64;
		fn get_minted_asset(asset: Vec<u8>) -> Vec<u8>;
		fn get_associated_assets(minted_asset: Vec<u8>) -> Vec<u8>;
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
		fn offchain_worker(block_number: T::BlockNumber) {
			let ad: AssetData = AssetData {};
			let local_store: StorageValueRef = StorageValueRef::persistent(b"collateral-reader::my-storage");
	
			const RECENTLY_SENT: () = ();
	
			let res = local_store.mutate(|last_send: Result<Option<T::BlockNumber>, StorageRetrievalError>| {
				match last_send {
					Ok(Some(block)) if block_number < block + T::GracePeriod::get() => {
						Err(RECENTLY_SENT)
					}
					_ => Ok(block_number),
				}
			});
	
			if let Err(MutateStorageError::ValueFunctionFailed(RECENTLY_SENT)) = res {
				return;
			}
	
			if let Err(MutateStorageError::ConcurrentModification(_)) = res {
				return;
			}
	
			if let Ok(block_number) = res {
				for asset in ad.get_supported_assets() {
					log::info!("assets: {:?}", asset);
					if let Err(e) = Self::send_signed(asset.clone()) {
						log::error!("Failed to submit asset stats for {:?}: {:?}", asset, e);
					}
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

		fn get_associated_assets(minted_asset: Vec<u8>) -> Vec<u8> {
			b"DOT".to_vec()
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn get_asset_stats)]
	pub(super) type AssetStatsStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::MaxVec>, AssetStats, OptionQuery>;

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

	impl<T: Config> Pallet<T> {
		fn send_signed(asset: Asset) -> Result<(), &'static str> {
			let signer = Signer::<T, T::AuthorityId>::all_accounts();
			if signer.can_sign() {
				let mut token: BoundedVec<u8, T::MaxVec> = BoundedVec::default();
				token.try_extend(asset.symbol.clone().into_iter());
				log::info!("asset {:?}", token.clone());
				let results = signer.send_signed_transaction(|_account| Call::save_asset_stats { token: token.clone() });
				for (acc, res) in &results {
					match res {
						Ok(()) => log::info!("[{:?}] Submitted Asset Stats:", acc.id),
 						Err(e) => log::error!("[{:?}] Failed to submit Asset Stats: {:?}", acc.id, e),
					}
				}
				Ok(())
			} else {
				Err("No local accounts available. Consider adding one via `author_insertKey` RPC.")
			}
		}
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn save_asset_stats(
			origin: OriginFor<T>,
			token: BoundedVec<u8, T::MaxVec>,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let asset = token.to_vec();
			let locked = T::AssetHelper::get_locked(asset.clone());
			let issued = T::AssetHelper::get_issued(asset.clone());
			let minted_asset = T::AssetHelper::get_minted_asset(asset.clone());
			<AssetStatsStorage<T>>::insert(
				token,
				AssetStats { asset: asset.clone(), issued, locked, minted_asset },
			);
			Ok(())
		}
	}
}
