use crate::{
	helper::helpers,
	mock::{new_test_ext, RuntimeEvent, *},
};
use frame_support::assert_ok;

use super::{AssetData, AssetStats};
use crate::{mock::MockInterlayData, pallet::AssetCollector};
use codec::Decode;
use frame_support::storage::bounded_vec::BoundedVec;
use sp_core::{offchain::testing, ConstU32, Pair};
use sp_keyring::AccountKeyring;
use sp_runtime::testing::TestXt;

type Extrinsic = TestXt<RuntimeCall, ()>;

#[test]
fn signed_transaction_on_chain() {
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();

	let mut t = new_offchain_test_ext(pool);

	t.execute_with(|| {
		let ad = AssetData {};
		TemplateModule::send_transactions(&ad.clone(), &ad.clone());

		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature.unwrap().0, 3);
	});
}

#[test]
fn test_save_asset_event() {
	let (pool, _pool_state) = testing::TestTransactionPoolExt::new();
	let alice = AccountKeyring::Alice.pair();
	let origin = frame_system::RawOrigin::Signed(alice.public());
	let mut t = new_offchain_test_ext(pool);
	let token: BoundedVec<u8, ConstU32<100>> = BoundedVec::default(); // Preferably use a more descriptive token

	t.execute_with(|| {
		System::set_block_number(1);
		let token_clone = token.clone();
		let asset_stats =
			AssetStats { asset: b"".to_vec(), locked: 0, issued: 0, minted_asset: b"".to_vec() };
		assert_ok!(TemplateModule::save_asset_stats(
			origin.into(),
			token_clone,
			asset_stats.clone()
		));

		// Ensure the event was emitted
		let _expected_token = token.clone();
		let expected_who = alice.public();
		assert!(System::events().iter().any(|record| match &record.event {
			RuntimeEvent::TemplateModule(crate::Event::AssetUpdated {
				token: event_token,
				who,
			}) => *event_token == token && *who == expected_who,
			_ => false,
		}));
	});
}

#[test]
fn save_asset_stats_works() {
	new_test_ext().execute_with(|| {
		let token: BoundedVec<u8, ConstU32<100>> = BoundedVec::default();
		let alice = AccountKeyring::Alice.pair();
		let origin = frame_system::RawOrigin::Signed(alice.public());
		let asset_stats = AssetStats {
			asset: token.clone().to_vec(),
			locked: 44,
			issued: 2,
			minted_asset: b"LDOT".to_vec(),
		};

		assert_ok!(TemplateModule::save_asset_stats(
			origin.into(),
			token.clone(),
			asset_stats.clone()
		));

		assert_eq!(TemplateModule::asset_stats(token.clone()), Some(asset_stats));
	});
}

#[test]
fn test_supported_assets() {
	let interlay_data = MockInterlayData {}; // Instantiate your data source
	let assets = interlay_data.supported_assets();
	assert_eq!(assets.len(), 1); // Assuming the mock implementation returns 1 asset
	                         // Add more assertions if necessary
}

#[test]
fn test_locked() {
	let interlay_data = MockInterlayData {};
	let asset = vec![1, 2, 3];
	let locked = interlay_data.locked(asset.clone());
	assert_eq!(locked, 123);
}

#[test]
fn test_issued() {
	let interlay_data = MockInterlayData {};
	let asset = vec![1, 2, 3];
	let issued = interlay_data.issued(asset.clone());
	assert_eq!(issued, 456);
}

#[test]
fn test_minted_asset() {
	let interlay_data = MockInterlayData {};
	let asset = vec![1, 2, 3];
	let minted_asset = interlay_data.minted_asset(asset.clone());
	assert_eq!(minted_asset, vec![4, 5, 6]);
}

#[test]
fn test_associated_assets() {
	let interlay_data = MockInterlayData {};
	let minted_asset = vec![4, 5, 6];
	let associated_assets = interlay_data.associated_assets(minted_asset.clone());
	assert_eq!(associated_assets, vec![7, 8, 9]);
}

#[test]
fn test_generate_storage_key() {
	let module_prefix = "Module";
	let storage_item_prefix = "StorageItem";

	let storage_key = helpers::generate_storage_key(module_prefix, storage_item_prefix);

	assert_eq!(storage_key.len(), 32);
	let hex_string = helpers::to_hex(storage_key.clone());
	assert_eq!(hex_string, "f7bc842de0628e5efb0481209c6551eef060e3619f9cd538c7d03fe5f89b9d4b");
}

#[test]
fn test_to_hex() {
	let storage_key = vec![0x12, 0x34, 0x56, 0x78];
	let hex_string = helpers::to_hex(storage_key.clone());

	assert_eq!(hex_string, "12345678");
}

#[test]
fn test_generate_double_storage_key() {
	let module_prefix = "Module";
	let storage_item_prefix = "StorageItem";
	let id = "123";

	let storage_key = helpers::generate_double_storage_key(module_prefix, storage_item_prefix, id);

	assert_eq!(storage_key.len(), 43);

	let hex_string = helpers::to_hex(storage_key.clone());
	assert_eq!(
		hex_string,
		"f7bc842de0628e5efb0481209c6551eef060e3619f9cd538c7d03fe5f89b9d4b85e8a73f227d693c313233"
	);
}

#[test]
fn test_generate_double_storage_keys() {
	let module_prefix = "Module";
	let storage_item_prefix = "StorageItem";
	let key1 = "Key1";
	let key2 = "Key2";

	let storage_key =
		helpers::generate_double_storage_keys(module_prefix, storage_item_prefix, key1, key2);

	assert_eq!(storage_key.len(), 56);

	let hex_string = helpers::to_hex(storage_key.clone());
	assert_eq!(hex_string, "f7bc842de0628e5efb0481209c6551eef060e3619f9cd538c7d03fe5f89b9d4b4d26e586dc10a1bc4b65793157f7f2fb894dfd264b657932");
}
