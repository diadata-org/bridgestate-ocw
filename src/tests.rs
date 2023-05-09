use crate::mock::*;

use codec::Decode;
use sp_core::offchain::{testing};

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
