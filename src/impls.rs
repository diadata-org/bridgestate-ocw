use crate::{Asset, AssetCollector, AssetData};

use sp_std::{ str, vec, vec::Vec, boxed::Box};

pub struct RPCHelper1 {}
pub struct RPCHelper2 {}

pub trait RPCCalls {
	fn get_supported_assets(&self) -> Result<Vec<Asset>, &'static str>;

	fn get_locked(&self, asset: Vec<u8>) -> Result<u128, &'static str>;

	fn get_issued(&self, asset: Vec<u8>) -> Result<u128, &'static str>;

	fn get_minted_asset(&self, asset: Vec<u8>) -> Result<Vec<u8>, &'static str>;

	fn get_associated_assets(&self, minted_asset: Vec<u8>) -> Result<Vec<u8>, &'static str>;
}

impl RPCCalls for RPCHelper1 {
	fn get_supported_assets(&self) -> Result<Vec<Asset>, &'static str> {
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

		Ok(assets)
	}

	fn get_locked(&self, _asset: Vec<u8>) -> Result<u128, &'static str> {
		Ok(45)
	}

	fn get_issued(&self, _asset: Vec<u8>) -> Result<u128, &'static str> {
		Ok(44)
	}

	fn get_minted_asset(&self, _asset: Vec<u8>) -> Result<Vec<u8>, &'static str> {
		Ok(b"LDOT".to_vec())
	}

	fn get_associated_assets(&self, _minted_asset: Vec<u8>) -> Result<Vec<u8>, &'static str> {
		Ok(b"DOT".to_vec())
	}
}

impl RPCCalls for RPCHelper2 {
	fn get_supported_assets(&self) -> Result<Vec<Asset>, &'static str> {
		Err("err")
	}

	fn get_locked(&self, _asset: Vec<u8>) -> Result<u128, &'static str> {
		Err("err")
	}

	fn get_issued(&self, _asset: Vec<u8>) -> Result<u128, &'static str> {
		Err("err")
	}

	fn get_minted_asset(&self, _asset: Vec<u8>) -> Result<Vec<u8>, &'static str> {
		Err("err")
	}

	fn get_associated_assets(&self, _minted_asset: Vec<u8>) -> Result<Vec<u8>, &'static str> {
		Err("err")
	}
}

impl AssetCollector for AssetData {
    
	fn get_supported_assets(&self) -> Vec<Asset> {
        let helpers: Vec<Box<dyn RPCCalls>> =
        vec![Box::new(RPCHelper1 {}), Box::new(RPCHelper2 {})];
	

		for helper in helpers {
			let result = helper.get_supported_assets();
			match result {
				Ok(assets) => return assets,
				Err(_e) => {
					// "Error occurred, retrying with the next helper..."
					continue
				},
			}
		}
		vec![Asset::default()]
	}

	fn get_locked(self,asset: Vec<u8>) -> u128 {
		let helpers: Vec<Box<dyn RPCCalls>> =
        vec![Box::new(RPCHelper1 {}), Box::new(RPCHelper2 {})];
	

		for helper in helpers {
			let result = helper.get_locked(asset.clone());
			match result {
				Ok(locked) => return locked,
				Err(_e) => {
					// "Error occurred, retrying with the next helper..."
					continue
				},
			}
		}
		0
	}
	fn get_issued(self, asset: Vec<u8>) -> u128 {
		let helpers: Vec<Box<dyn RPCCalls>> =
        vec![Box::new(RPCHelper1 {}), Box::new(RPCHelper2 {})];
	

		for helper in helpers {
			let result = helper.get_issued(asset.clone());
			match result {
				Ok(issued) => return issued,
				Err(_e) => {
					// "Error occurred, retrying with the next helper..."
					continue
				},
			}
		}
		0
	}
	fn get_minted_asset(self,asset: Vec<u8>) -> Vec<u8> {
		let helpers: Vec<Box<dyn RPCCalls>> =
        vec![Box::new(RPCHelper1 {}), Box::new(RPCHelper2 {})];
	

		for helper in helpers {
			let result = helper.get_minted_asset(asset.clone());
			match result {
				Ok(mintedasset) => return mintedasset,
				Err(_e) => {
					// "Error occurred, retrying with the next helper..."
					continue
				},
			}
		}
		vec![0]
	}

	fn get_associated_assets(minted_asset: Vec<u8>) -> Vec<u8> {
        let helpers: Vec<Box<dyn RPCCalls>> =
        vec![Box::new(RPCHelper1 {}), Box::new(RPCHelper2 {})];
	

		for helper in helpers {
			let result = helper.get_associated_assets(minted_asset.clone());
			match result {
				Ok(assets) => return assets,
				Err(_e) => {
					// "Error occurred, retrying with the next helper..."
					continue
				},
			}
		}
		vec![0]
	}
}
