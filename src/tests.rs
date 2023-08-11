use crate::mock::{new_test_ext, RuntimeEvent, *};

use codec::Decode;
use frame_support::storage::bounded_vec::BoundedVec;
use sp_core::{offchain::testing, ConstU32, Pair};
use sp_keyring::AccountKeyring;

use sp_core::storage::StorageKey;
use sp_runtime::offchain::http;
use std::str::from_utf8;

use sp_runtime::testing::TestXt;

type Extrinsic = TestXt<RuntimeCall, ()>;

use super::helper::Helper;
use super::AssetStats;

#[test]
fn signed_transaction_on_chain() {
	let (pool, pool_state) = testing::TestTransactionPoolExt::new();

	let mut t = new_offchain_test_ext(pool);

	t.execute_with(|| {
		TemplateModule::send_transactions();

		let tx = pool_state.write().transactions.pop().unwrap();
		let tx = Extrinsic::decode(&mut &*tx).unwrap();
		assert_eq!(tx.signature.unwrap().0, 2);
	});
}

#[test]
fn test_save_asset_event() {
	let (pool, _pool_state) = testing::TestTransactionPoolExt::new();
	let alice = AccountKeyring::Alice.pair();
	let origin = frame_system::RawOrigin::Signed(alice.public());
	let mut t = new_offchain_test_ext(pool);
	let token: BoundedVec<u8, ConstU32<100>> = BoundedVec::default(); // Preferably use a more descriptive token
	let asset_stats =
		AssetStats { asset: b"".to_vec(), locked: 0, issued: 0, minted_asset: b"".to_vec() };
	t.execute_with(|| {
		System::set_block_number(1);
		let token_clone = token.clone();
		TemplateModule::save_asset_stats(origin.into(), token_clone, asset_stats);

		// Ensure the event was emitted
		let expected_token = token.clone();
		let expected_who = alice.public().clone();
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
		let mut asset_stats = AssetStats {
			asset: token.clone().to_vec(),
			locked: 44,
			issued: 2,
			minted_asset: b"LDOT".to_vec(),
		};

		TemplateModule::save_asset_stats(origin.into(), token.clone(), asset_stats.clone());

		assert_eq!(TemplateModule::get_asset_stats(token.clone()), Some(asset_stats));
	});
}

#[test]
fn test_generate_storage_key() {
	let module_prefix: &str = "VaultRegistry";
	let storage_item_prefix: &str = "TotalUserVaultCollateral";

	let storage_key = Helper::generate_storage_key(module_prefix, storage_item_prefix);

	let hex_string: String = storage_key
		.iter()
		.map(|byte| format!("{:02x}", byte))
		.collect::<Vec<String>>()
		.join("");

	println!("Hexadecimal: {}", hex_string);
}

#[test]
fn test_generate_double_storage_key() {
	let module_prefix: &str = "Tokens";
	let storage_item_prefix: &str = "TotalIssuance";
	let _item_prefix: &str = "Token";
	let _1_item_prefix: &str = "DOT";

	let storage_key = Helper::generate_double_storage_keys(
		module_prefix,
		storage_item_prefix,
		_item_prefix,
		_1_item_prefix,
	);

	let hex_string: String = storage_key
		.iter()
		.map(|byte| format!("{:02x}", byte))
		.collect::<Vec<String>>()
		.join("");

	println!("Hexadecimal: {}", hex_string);
}

#[test]
fn test_fetch_data() {
	let hex = "70ce0360818118000000000000000000";

	let big_endian_value: u128 = u128::from_str_radix(hex, 16).unwrap();

	// Convert the value to little endian
	// let little_endian_value = big_endian_value.to_le();
	let little_endian_value = big_endian_value.swap_bytes();

	println!("The little endian u128 value is: {}", little_endian_value);
}

#[test]
fn test_total_user_vault_collateral() {
	let r = Helper::total_user_vault_collateral();

	println!("collateral: {}", r);
}
