#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod helper;
pub mod impls;
pub mod interlay;
pub mod multichain;

use frame_support::{sp_runtime::KeyTypeId, traits::Get};
use frame_system::{
	offchain::{CreateSignedTransaction, SendSignedTransaction},
	pallet_prelude::BlockNumberFor,
};
pub use pallet::*;
use sp_runtime::offchain::storage::{MutateStorageError, StorageRetrievalError, StorageValueRef};

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"dia!");
pub mod crypto {
	use sp_core::sr25519::Signature as Sr25519Signature;

	use crate::KEY_TYPE;
	use frame_support::sp_runtime::{
		app_crypto::{app_crypto, sr25519},
		traits::Verify,
		MultiSignature, MultiSigner,
	};

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
	use sp_std::{str, vec, vec::Vec};

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
		type GracePeriod: Get<BlockNumberFor<Self>>;

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
		fn supported_assets(&self) -> Vec<Asset>;

		/// Returns the amount of the specified asset that is currently locked.
		///
		/// # Parameters
		///
		/// - `asset`: A `Vec<u8>` that uniquely identifies an asset.
		///
		/// # Returns
		///
		/// A `u64` value representing the amount of the asset that is locked.
		fn locked(self, asset: Vec<u8>) -> u128;

		/// Returns the total issued amount of the specified asset.
		///
		/// # Parameters
		///
		/// - `asset`: A `Vec<u8>` that uniquely identifies an asset.
		///
		/// # Returns
		///
		/// A `u64` value representing the total issued amount of the asset.
		fn issued(self, asset: Vec<u8>) -> u128;

		/// Returns the minted asset associated with the specified asset.
		///
		/// # Parameters
		///
		/// - `asset`: A `Vec<u8>` that uniquely identifies an asset.
		///
		/// # Returns
		///
		/// A `Vec<u8>` representing the minted asset associated with the input asset.
		fn minted_asset(self, asset: Vec<u8>) -> Vec<u8>;

		/// Returns the assets associated with the specified minted asset.
		///
		/// # Parameters
		///
		/// - `minted_asset`: A `Vec<u8>` that uniquely identifies a minted asset.
		///
		/// # Returns
		///
		/// A `Vec<u8>` representing the assets associated with the minted asset.
		fn associated_assets(self, minted_asset: Vec<u8>) -> Vec<u8>;
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
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct AssetData {}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct MultichainData {}

	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct InterlayData {}

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
		minted_amount: u128,
	}

	/// Represents the statistics for an asset.
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, Default, TypeInfo)]
	pub struct AssetStats {
		/// The asset for which the statistics are being recorded.
		pub asset: Vec<u8>,
		/// The total amount of the asset that has been issued.
		pub issued: u128,
		/// The total amount of the asset that is currently locked.
		pub locked: u128,
		/// The asset that has been minted in relation to the original asset.
		pub minted_asset: Vec<u8>,
	}

	// Define the hooks trait implementation for off-chain worker
	// This trait is called when an off-chain worker is triggered.

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn offchain_worker(block_number: BlockNumberFor<T>) {
			Self::start(block_number);
		}
	}
	// Implement the main functionality of the pallet

	impl<T: Config> Pallet<T> {
		/// Start the asset statistics collection process.
		/// This function is called by the off-chain worker.
		pub fn start(block_number: BlockNumberFor<T>) {
			let local_store: StorageValueRef =
				StorageValueRef::persistent(b"collateral-reader::my-storage");

			const RECENTLY_SENT: () = ();

			let res = local_store.mutate(
				|last_send: Result<Option<BlockNumberFor<T>>, StorageRetrievalError>| {
					match last_send {
						Ok(Some(block)) if block_number < block + T::GracePeriod::get() =>
							Err(RECENTLY_SENT),
						_ => Ok(block_number),
					}
				},
			);

			if let Err(MutateStorageError::ValueFunctionFailed(RECENTLY_SENT)) = res {
				return
			}

			if let Err(MutateStorageError::ConcurrentModification(_)) = res {
				return
			}

			if let Ok(_) = res {
				let id: InterlayData = InterlayData {};
				let md: MultichainData = MultichainData {};

				Self::send_transactions(&id.clone(), &md.clone());
			}
		}

		/// Send asset statistics transactions .
		/// This function sends commits asset statistics to chain.
		pub fn send_transactions(
			inter: &(impl AssetCollector + Clone),
			md: &(impl AssetCollector + Clone),
		) {
			for asset in inter.supported_assets() {
				log::info!("InterlayData: {:?}", asset);

				let asset_stats = AssetStats {
					asset: asset.symbol.clone(),
					locked: inter.clone().locked(asset.clone().symbol),
					issued: inter.clone().issued(asset.clone().symbol),
					minted_asset: inter.clone().minted_asset(asset.clone().symbol),
				};

				if let Err(e) = Self::send_signed(asset.clone(), asset_stats.clone()) {
					log::error!("Failed to submit InterlayData stats for {:?}: {:?}", asset, e);
				}
			}
			for asset in md.supported_assets() {
				let asset_id = asset.symbol.clone();
				log::info!("assets: {:?}", asset);
				let asset_stats = AssetStats {
					asset: asset_id.clone(),
					locked: md.clone().locked(asset_id.clone()),
					issued: md.clone().issued(asset_id.clone()),
					minted_asset: md.clone().minted_asset(asset_id.clone()),
				};
				if let Err(e) = Self::send_signed_multichain(asset.clone(), asset_stats) {
					log::error!("Failed to submit asset stats for {:?}: {:?}", asset, e);
				}
			}
		}
	}

	// Define the storage structure for asset statistics
	#[pallet::storage]
	#[pallet::getter(fn asset_stats)]
	pub(super) type AssetStatsStorage<T: Config> =
		StorageMap<_, Blake2_128Concat, BoundedVec<u8, T::MaxVec>, AssetStats, OptionQuery>;

	// Define the events generated by the pallet
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		AssetUpdated { token: BoundedVec<u8, T::MaxVec>, who: T::AccountId },
	}
	// Implement additional functions for the pallet
	impl<T: Config> Pallet<T> {
		fn send_signed(asset: Asset, asset_stats: AssetStats) -> Result<(), &'static str> {
			let signer = Signer::<T, T::AuthorityId>::all_accounts();
			if signer.can_sign() {
				let mut token: BoundedVec<u8, T::MaxVec> = BoundedVec::default();
				token.try_extend(asset.symbol.clone().into_iter()).expect("token vec");
				log::info!("asset {:?}", token.clone());
				let results = signer.send_signed_transaction(|_account| Call::save_asset_stats {
					token: token.clone(),
					asset_stats: asset_stats.clone(),
				});
				for (acc, res) in &results {
					match res {
						Ok(()) => {
							log::info!("[{:?}] Submitted Asset Stats:", acc.id)
						},
						Err(e) => {
							log::error!("[{:?}] Failed to submit Asset Stats: {:?}", acc.id, e)
						},
					}
				}
				Ok(())
			} else {
				Err("No local accounts available. Consider adding one via `author_insertKey` RPC.")
			}
		}

		fn send_signed_multichain(
			asset: Asset,
			asset_stats: AssetStats,
		) -> Result<(), &'static str> {
			let signer = Signer::<T, T::AuthorityId>::all_accounts();
			if signer.can_sign() {
				let mut token: BoundedVec<u8, T::MaxVec> = BoundedVec::default();
				token.try_extend(asset.symbol.clone().into_iter()).expect("token vec");
				log::info!("asset {:?}", token.clone());
				let results =
					signer.send_signed_transaction(|_account| Call::save_multichain_asset_stats {
						token: token.clone(),
						asset_stats: asset_stats.clone(),
					});
				for (acc, res) in &results {
					match res {
						Ok(()) => {
							log::info!("[{:?}] Submitted Asset Stats:", acc.id)
						},
						Err(e) => {
							log::error!("[{:?}] Failed to submit Asset Stats: {:?}", acc.id, e)
						},
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
		/// Save asset statistics to the current chain.
		///
		/// This function allows authorized accounts to save asset statistics to current chain.
		/// The provided asset statistics will be associated with the given asset token and stored
		/// in the pallet's storage.
		///
		/// - `origin`: The origin of the call. This should be a signed account.
		/// - `token`: The asset token for which the statistics are being saved.
		/// - `asset_stats`: The asset statistics data to be saved.
		///
		/// Returns `Ok(())` on success or an error if the operation fails.
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn save_asset_stats(
			origin: OriginFor<T>,
			token: BoundedVec<u8, T::MaxVec>,
			asset_stats: AssetStats,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let asset = token.to_vec();
			let locked = asset_stats.locked;
			let issued = asset_stats.issued;
			let minted_asset = asset_stats.minted_asset;

			Self::deposit_event(Event::AssetUpdated { token: token.clone(), who });

			<AssetStatsStorage<T>>::insert(
				token,
				AssetStats { asset: asset.clone(), issued, locked, minted_asset },
			);

			Ok(())
		}

		/// Save asset statistics of multichain to the current chain.
		///
		/// This function allows authorized accounts to save asset statistics of multichain .
		/// The provided asset statistics will be associated with the given asset token and stored
		/// in the pallet's storage.
		///
		/// - `origin`: The origin of the call. This should be a signed account.
		/// - `token`: The asset token for which the statistics are being saved.
		/// - `asset_stats`: The asset statistics data to be saved.
		///
		/// Returns `Ok(())` on success or an error if the operation fails.

		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1).ref_time())]
		pub fn save_multichain_asset_stats(
			origin: OriginFor<T>,
			token: BoundedVec<u8, T::MaxVec>,
			asset_stats: AssetStats,
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			Self::deposit_event(Event::AssetUpdated { token: token.clone(), who });
			log::info!("Saving asset stats {:?}", asset_stats.clone());
			<AssetStatsStorage<T>>::insert(token, asset_stats);

			Ok(())
		}
	}
}
