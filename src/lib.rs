#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod impls;

use frame_support::{sp_runtime::KeyTypeId, traits::Get};
use frame_system::offchain::{CreateSignedTransaction, SendSignedTransaction};
pub use pallet::*;
use sp_runtime::offchain::storage::{MutateStorageError, StorageRetrievalError, StorageValueRef};

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"dia!");

pub mod crypto {
	use sp_core::sr25519::Signature as Sr25519Signature;

	use super::KEY_TYPE;
	use frame_support::sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
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
	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for TestAuthId
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use codec::{Decode, Encode};
	use frame_support::pallet_prelude::*;
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
	/// The `Config` trait provides the types and constants 
	pub trait Config: CreateSignedTransaction<Call<Self>> + frame_system::Config {
		/// The identifier type for an offchain worker.
		type AuthorityId: AppCrypto<Self::Public, Self::Signature>;

		/// Helper type for retriving asset stats.
		type AssetHelper: AssetCollector;

		/// Specifies the maximum length of a vector that can be
		/// stored in a bounded vector type.
		#[pallet::constant]
		type MaxVec: Get<u32>;

		/// Specifies the number of blocks during which an action must
		/// be taken before some consequence is incurred.
		#[pallet::constant]
		type GracePeriod: Get<Self::BlockNumber>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
	}

	/// The `AssetCollector` trait defines a set of functions for interacting with
	/// assets in a blockchain context.
	pub trait AssetCollector {
		/// Returns a list of all assets supported by the chain.
		///
		/// # Returns
		///
		/// A `Vec<Asset>` holding all supported assets.
		fn get_supported_assets(&self) -> Vec<Asset>;

		/// Returns the amount of the specified asset that is currently locked.
		///
		/// # Parameters
		///
		/// - `asset`: A `Vec<u8>` that uniquely identifies an asset.
		///
		/// # Returns
		///
		/// A `u64` value representing the amount of the asset that is locked.
		fn get_locked(asset: Vec<u8>) -> u64;

		/// Returns the total issued amount of the specified asset.
		///
		/// # Parameters
		///
		/// - `asset`: A `Vec<u8>` that uniquely identifies an asset.
		///
		/// # Returns
		///
		/// A `u64` value representing the total issued amount of the asset.
		fn get_issued(asset: Vec<u8>) -> u64;

		/// Returns the minted asset associated with the specified asset.
		///
		/// # Parameters
		///
		/// - `asset`: A `Vec<u8>` that uniquely identifies an asset.
		///
		/// # Returns
		///
		/// A `Vec<u8>` representing the minted asset associated with the input asset.
		fn get_minted_asset(asset: Vec<u8>) -> Vec<u8>;

		/// Returns the assets associated with the specified minted asset.
		///
		/// # Parameters
		///
		/// - `minted_asset`: A `Vec<u8>` that uniquely identifies a minted asset.
		///
		/// # Returns
		///
		/// A `Vec<u8>` representing the assets associated with the minted asset.
		fn get_associated_assets(minted_asset: Vec<u8>) -> Vec<u8>;
	}

	/// Represents an asset in the system.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct Asset {
		/// The address of the asset.
		pub address: Vec<u8>,
		/// Additional metadata associated with the asset.
		/// This field is optional.
		pub metadata: Vec<u8>,
		/// The chain on which the asset exists.
		pub chain: Vec<u8>,
		/// The number of decimal places used by the asset.
		pub decimals: u64,
		/// The symbol of the asset.
		pub symbol: Vec<u8>,
		/// The name of the asset.
		pub name: Vec<u8>,
	}

	/// Data associated with an asset.
	/// Currently, this structure is empty.
	pub struct AssetData {}

	/// Represents a token name in the system.
	/// Currently, this is represented as a `u32`.
	pub type TokenName = u32;

	/// Represents a set of assets managed by a bridge.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct BridgeSet {
		/// Assets that are locked by the bridge.
		locked_assets: Vec<Asset>,
		/// The asset that has been minted by the bridge.
		minted_asset: Asset,
		/// The amount of the minted asset.
		minted_amount: u64,
	}

	/// Represents the statistics for an asset.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct AssetStats {
		/// The asset for which the statistics are being recorded.
		pub asset: Vec<u8>,
		/// The total amount of the asset that has been issued.
		pub issued: u64,
		/// The total amount of the asset that is currently locked.
		pub locked: u64,
		/// The asset that has been minted in relation to the original asset.
		pub minted_asset: Vec<u8>,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: T::BlockNumber) {
			Self::start(block_number);
		}
	}
	impl<T: Config> Pallet<T> {
		pub fn start(block_number: T::BlockNumber) {
			let local_store: StorageValueRef =
				StorageValueRef::persistent(b"collateral-reader::my-storage");

			const RECENTLY_SENT: () = ();

			let res = local_store.mutate(
				|last_send: Result<Option<T::BlockNumber>, StorageRetrievalError>| match last_send {
					Ok(Some(block)) if block_number < block + T::GracePeriod::get() =>
						Err(RECENTLY_SENT),
					_ => Ok(block_number),
				},
			);

			if let Err(MutateStorageError::ValueFunctionFailed(RECENTLY_SENT)) = res {
				return
			}

			if let Err(MutateStorageError::ConcurrentModification(_)) = res {
				return
			}

			if let Ok(_) = res {
				Self::send_transactions();
			}
		}

		pub fn send_transactions() {
			let ad: AssetData = AssetData {};
			for asset in ad.get_supported_assets() {
				log::info!("assets: {:?}", asset);
				if let Err(e) = Self::send_signed(asset.clone()) {
					log::error!("Failed to submit asset stats for {:?}: {:?}", asset, e);
				}
			}
		}
	}

	#[pallet::storage]
	#[pallet::getter(fn get_asset_stats)]
	pub(super) type AssetStatsStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::MaxVec>, AssetStats, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AssetUpdated { token: BoundedVec<u8, T::MaxVec>, who: T::AccountId },
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
				token.try_extend(asset.symbol.clone().into_iter()).unwrap();
				log::info!("asset {:?}", token.clone());
				let results = signer.send_signed_transaction(|_account| Call::save_asset_stats {
					token: token.clone(),
				});
				for (acc, res) in &results {
					match res {
						Ok(()) => {
							log::info!("[{:?}] Submitted Asset Stats:", acc.id)
						},
						Err(e) =>
							log::error!("[{:?}] Failed to submit Asset Stats: {:?}", acc.id, e),
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

			Self::deposit_event(Event::AssetUpdated { token: token.clone(), who });

			<AssetStatsStorage<T>>::insert(
				token,
				AssetStats { asset: asset.clone(), issued, locked, minted_asset },
			);

			Ok(())
		}
	}
}
