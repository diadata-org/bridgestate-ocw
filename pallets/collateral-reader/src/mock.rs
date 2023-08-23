use crate as pallet_template;
use crate as pallet_collateral_reader;
use crate::{Asset, AssetCollector};
use frame_support::traits::{ConstU16, ConstU32, ConstU64};

use sp_core::{offchain::testing::TestTransactionPoolExt, H256};
use sp_runtime::{
	traits,
	traits::{BlakeTwo256, IdentityLookup},
	BuildStorage,
};

pub struct MockInterlayData;
use sp_runtime::traits::{IdentifyAccount, Verify};

use sp_runtime::{testing::TestXt, traits::Extrinsic as ExtrinsicT, RuntimeAppPublic};

use sp_core::{
	offchain::{testing, OffchainWorkerExt, TransactionPoolExt},
	sr25519::Signature,
};
use sp_keystore::{testing::MemoryKeystore, Keystore, KeystoreExt};

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		TemplateModule: pallet_collateral_reader,
	}
);

impl frame_system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Nonce = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	// type AccountId = u64;
	type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = Block;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = ();
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_template::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type AssetHelper = pallet_template::AssetData;
	type AuthorityId = pallet_template::crypto::TestAuthId;
	type MaxVec = ConstU32<100>;
	type GracePeriod = ConstU64<10>;
}

impl frame_system::offchain::SigningTypes for Test {
	type Public = <Signature as traits::Verify>::Signer;
	type Signature = Signature;
}
type Extrinsic = TestXt<RuntimeCall, ()>;
type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

impl<LocalCall> frame_system::offchain::SendTransactionTypes<LocalCall> for Test
where
	RuntimeCall: From<LocalCall>,
{
	type OverarchingCall = RuntimeCall;
	type Extrinsic = Extrinsic;
}

impl<LocalCall> frame_system::offchain::CreateSignedTransaction<LocalCall> for Test
where
	RuntimeCall: From<LocalCall>,
{
	fn create_transaction<C: frame_system::offchain::AppCrypto<Self::Public, Self::Signature>>(
		call: RuntimeCall,
		_public: <Signature as Verify>::Signer,
		_account: AccountId,
		nonce: u64,
	) -> Option<(RuntimeCall, <Extrinsic as ExtrinsicT>::SignaturePayload)> {
		Some((call, (nonce, ())))
	}
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	frame_system::GenesisConfig::<Test>::default().build_storage().unwrap().into()
}

pub fn new_offchain_test_ext(pool: TestTransactionPoolExt) -> sp_io::TestExternalities {
	const PHRASE: &str =
		"news slush supreme milk chapter athlete soap sausage put clutch what kitten";

	let (offchain, _) = testing::TestOffchainExt::new();
	let keystore = MemoryKeystore::new();
	keystore
		.sr25519_generate_new(crate::crypto::Public::ID, Some(&format!("{}/dia", PHRASE)))
		.unwrap();

	let mut t = sp_io::TestExternalities::default();
	t.register_extension(OffchainWorkerExt::new(offchain));
	t.register_extension(TransactionPoolExt::new(pool));
	t.register_extension(KeystoreExt::new(keystore));
	t
}

impl AssetCollector for MockInterlayData {
	fn supported_assets(&self) -> Vec<Asset> {
		// Return a mock list of supported assets
		vec![
			Asset {
				address: vec![1],
				chain: vec![2],
				metadata: vec![3],
				decimals: 0,
				symbol: vec![4],
				name: vec![5],
			},
			// Add more mock assets if needed
		]
	}

	fn locked(self, _asset: Vec<u8>) -> u128 {
		// Return a mock locked amount
		123
	}

	fn issued(self, _asset: Vec<u8>) -> u128 {
		// Return a mock issued amount
		456
	}

	fn minted_asset(self, _asset: Vec<u8>) -> Vec<u8> {
		// Return a mock minted asset
		vec![4, 5, 6]
	}

	fn associated_assets(self, _minted_asset: Vec<u8>) -> Vec<u8> {
		// Return a mock associated asset
		vec![7, 8, 9]
	}
}
