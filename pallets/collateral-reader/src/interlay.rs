use crate::{Asset, AssetCollector, InterlayData};

use crate::impls::{RPCCalls, RPCHelper1};

use serde::{Deserialize, Serialize};

use sp_std::borrow::ToOwned;

use scale_info::prelude::string::String;

use sp_std::{boxed::Box, str, vec, vec::Vec};

use crate::helper::helper;

/// Represents a JSON-RPC response structure.
#[derive(Serialize, Deserialize)]
struct RpcResponse {
	jsonrpc: String,
	result: Option<String>,
	id: String,
}

/// Helper implementation for making RPC calls specific to Interlay.
pub struct InterlayRPCHelper1 {}

impl RPCCalls for InterlayRPCHelper1 {
	/// Get the list of supported assets on Interlay.
	fn supported_assets(&self) -> Result<Vec<Asset>, &'static str> {
		let mut assets: Vec<Asset> = Vec::new();
		assets.push(Asset {
			address: b"2".to_vec(),
			chain: b"interlay".to_vec(),
			metadata: b"d67c5ba80ba065480001".to_vec(),
			decimals: 0,
			symbol: b"DOT".to_vec(),
			name: b"DOT".to_vec(),
		});

		Ok(assets)
	}
	/// Get the locked amount of a specific asset on Interlay.
	fn locked(&self, _asset: Vec<u8>) -> Result<u128, &'static str> {
		log::info!("calling get_locked");

		let module_name = "Tokens";
		let storage_name = "TotalIssuance";

		let storage_key = helper::generate_storage_key(module_name, &storage_name);

		let mut storage_key_hash = helper::to_hex(storage_key);

		storage_key_hash = "0x".to_owned() + &storage_key_hash + "d67c5ba80ba065480001";

		let result = helper::fetch_data("state_getKeys", &storage_key_hash);
		let mut locked = 0;
		match result {
			Ok(bytes) => {
				// Try to convert the bytes to a string.

				let json = String::from_utf8(bytes).unwrap();
				let parsed_data: RpcResponse = serde_json::from_str(&json).unwrap();

				match parsed_data.result {
					Some(res) => {
						log::error!("Result: {}", res);
						let stripped_string = res.strip_prefix("0x").unwrap_or(&res);

						locked = helper::hex_to_balance(&stripped_string);

						log::info!("Result: locked {}", locked)
					},
					None => log::error!("Result is null"),
				}
			},
			Err(_e) => {
				log::error!("HTTP error: ");
			},
		};

		Ok(locked)
	}

	/// Get the issued amount of a specific asset on Interlay.
	fn issued(&self, _asset: Vec<u8>) -> Result<u128, &'static str> {
		let issued_dot = helper::total_user_vault_collateral("DOT");
		let issued_usdt = helper::total_user_vault_collateral("USDT");

		log::info!("Issued dot: {}", issued_dot);
		log::info!("Issued usdt: {}", issued_usdt);

		let oracle_dot: u128 = helper::oracle("DOT");

		let oracle_usdt: u128 = helper::oracle("USDT");

		log::info!("oracle dot: {}", oracle_dot);
		log::info!("oracle usdt: {}", oracle_usdt);

		log::info!("backable dot: {}", issued_dot / (oracle_dot / 100000000000000000000));
		log::info!("backable usdt: {}", issued_usdt / (oracle_usdt / 100000000000000000000));

		let total_backable = issued_dot / (oracle_dot / 100000000000000000000) +
			issued_usdt / (oracle_usdt / 100000000000000000000);

		Ok(total_backable / 100)
	}
	/// Get the minted asset on Interlay.
	fn minted_asset(&self, _asset: Vec<u8>) -> Result<Vec<u8>, &'static str> {
		Ok(b"IBTC".to_vec())
	}
	/// Get the associated assets for a minted asset on Interlay.
	fn associated_assets(&self, minted_asset: Vec<u8>) -> Result<Vec<u8>, &'static str> {
		match minted_asset.as_slice() {
			b"IBTC" => Ok(b"DOT".to_vec()),
			_ => Err("Minted asset not recognized"),
		}
	}
}

impl AssetCollector for InterlayData {
	/// Get the list of supported assets using available RPC helpers.
	fn supported_assets(&self) -> Vec<Asset> {
		let helpers: Vec<Box<dyn RPCCalls>> = vec![Box::new(InterlayRPCHelper1 {})];

		for helper in helpers {
			let result = helper.supported_assets();
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
	/// Get the locked amount of an asset using available RPC helpers.

	fn locked(self, asset: Vec<u8>) -> u128 {
		let helpers: Vec<Box<dyn RPCCalls>> =
			vec![Box::new(InterlayRPCHelper1 {}), Box::new(InterlayRPCHelper1 {})];

		for helper in helpers {
			let result = helper.locked(asset.clone());
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
	/// Get the issued amount of an asset using available RPC helpers.

	fn issued(self, asset: Vec<u8>) -> u128 {
		let helpers: Vec<Box<dyn RPCCalls>> = vec![Box::new(InterlayRPCHelper1 {})];

		for helper in helpers {
			let result = helper.issued(asset.clone());
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
	/// Get the minted asset using available RPC helpers.

	fn minted_asset(self, asset: Vec<u8>) -> Vec<u8> {
		let helpers: Vec<Box<dyn RPCCalls>> = vec![Box::new(InterlayRPCHelper1 {})];

		for helper in helpers {
			let result = helper.minted_asset(asset.clone());
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

	/// Get the associated assets for a minted asset using available RPC helpers.
	fn associated_assets(self, minted_asset: Vec<u8>) -> Vec<u8> {
		let helpers: Vec<Box<dyn RPCCalls>> =
			vec![Box::new(RPCHelper1 {}), Box::new(InterlayRPCHelper1 {})];

		for helper in helpers {
			let result = helper.associated_assets(minted_asset.clone());
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
