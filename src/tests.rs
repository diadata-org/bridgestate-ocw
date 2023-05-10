use crate::mock::{new_test_ext, RuntimeEvent, *};

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

	t.execute_with(|| {
		System::set_block_number(1);
		let token_clone = token.clone();
		TemplateModule::save_asset_stats(origin.into(), token_clone);

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

		 TemplateModule::save_asset_stats(origin.into(), token.clone());

		assert_eq!(
			TemplateModule::get_asset_stats(token.clone()),
			Some(crate::AssetStats {
				asset: token.clone().to_vec(),
				issued: 44,
				locked: 45,
				minted_asset: b"LDOT".to_vec(),
			})
		);

});
}
