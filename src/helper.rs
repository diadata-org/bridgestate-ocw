pub mod Helper {
	use crate::alloc::borrow::ToOwned;
	use codec::{Decode, Input};
	use frame_support::{StorageHasher, Twox128, Twox64Concat};
	use scale_info::prelude::format;
	use scale_info::prelude::string::String;
	use scale_info::prelude::vec;
	use serde::{Deserialize, Serialize};
	use serde_json::{json, to_value, Value};
	use sp_core::storage::StorageKey;
	use sp_runtime::offchain::{http, Duration};
	use sp_std::vec::Vec;

	#[derive(Serialize, Deserialize)]
	struct RpcResponse {
		jsonrpc: String,
		result: Option<String>,
		id: String,
	}

	pub fn generate_storage_key(module_prefix: &str, storage_item_prefix: &str) -> Vec<u8> {
		let module_hash = Twox128::hash(module_prefix.as_bytes());
		let storage_hash = Twox128::hash(storage_item_prefix.as_bytes());

		let mut storage_key: Vec<_> = Vec::with_capacity(module_hash.len() + storage_hash.len());
		storage_key.extend_from_slice(&module_hash[..]);
		storage_key.extend_from_slice(&storage_hash[..]);

		storage_key
	}

	pub fn to_hex(storage_key: Vec<u8>) -> String {
		let hex_string: String = storage_key
			.iter()
			.map(|byte| format!("{:02x}", byte))
			.collect::<Vec<String>>()
			.join("");
		hex_string
	}

	// pub fn storage_map_final_key<H: StorageHasher>(
	//     pallet_prefix: &str,
	//     map_name: &str,
	//     key: &[u8],
	// ) -> StorageKey {
	//     let key_hashed = H::hash(key);
	//     let pallet_prefix_hashed = frame_support::Twox128::hash(pallet_prefix.as_bytes());
	//     let storage_prefix_hashed = frame_support::Twox128::hash(map_name.as_bytes());

	//     let mut final_key = Vec::with_capacity(
	//         pallet_prefix_hashed.len() + storage_prefix_hashed.len() + key_hashed.as_ref().len(),
	//     );

	//     final_key.extend_from_slice(&pallet_prefix_hashed[..]);
	//     final_key.extend_from_slice(&storage_prefix_hashed[..]);
	//     final_key.extend_from_slice(key_hashed.as_ref());

	//     StorageKey(final_key)
	// }

	pub fn generate_double_storage_key(
		module_prefix: &str,
		storage_item_prefix: &str,
		id: &str,
	) -> Vec<u8> {
		let mut storage_key = Vec::new();
		let module_hash = Twox128::hash(module_prefix.as_bytes());
		let storage_hash = Twox128::hash(storage_item_prefix.as_bytes());
		let id_hash = Twox64Concat::hash(id.as_bytes());

		storage_key.extend_from_slice(&module_hash[..]);
		storage_key.extend_from_slice(&storage_hash[..]);
		storage_key.extend_from_slice(&id_hash.as_ref());

		storage_key
	}

	pub fn generate_double_storage_keys(
		module_prefix: &str,
		storage_item_prefix: &str,
		key1: &str,
		key2: &str,
	) -> Vec<u8> {
		let mut storage_key = Vec::new();
		let module_hash = Twox128::hash(module_prefix.as_bytes());
		let storage_hash = Twox128::hash(storage_item_prefix.as_bytes());
		let key1_hash = Twox64Concat::hash(key1.as_bytes());
		let key2_hash = Twox64Concat::hash(key2.as_bytes());

		storage_key.extend_from_slice(&module_hash[..]);
		storage_key.extend_from_slice(&storage_hash[..]);
		storage_key.extend_from_slice(&key1_hash.as_ref());
		storage_key.extend_from_slice(&key2_hash.as_ref());

		storage_key
	}

	pub fn fetch_data(method: &str, param: &str) -> Result<Vec<u8>, http::Error> {
		let rpc_url = "https://interlay.api.onfinality.io/public";
		let mut rpc_request = "{
            \"jsonrpc\": \"2.0\",
            \"id\": \"1\",
            \"method\": \"state_getStorage\",
            \"params\": [\"passparam\"]
        }"
		.to_owned();

		rpc_request = rpc_request.replace("passparam", param);

		log::info!("request {}", rpc_request);

		let rbody = rpc_request.as_bytes();

		let deadline = sp_io::offchain::timestamp().add(Duration::from_millis(5_000));
		let request = http::Request::default()
			.method(http::Method::Post)
			.url(rpc_url)
			.body(vec![rbody])
			.add_header("Content-Type", "application/json");

		let pending = request.deadline(deadline).send().map_err(|_| http::Error::IoError)?;
		let response = pending.try_wait(deadline).map_err(|_| http::Error::DeadlineReached)??;
		if response.code != 200 {
			log::info!("Unexpected status code: {}", response.code);
			return Err(http::Error::Unknown);
		}

		let body = response.body().collect::<Vec<u8>>();
		let body_str = sp_runtime::sp_std::str::from_utf8(&body).map_err(|_| {
			log::info!("No UTF8 body");
			http::Error::Unknown
		})?;

		log::info!("body_str: {:?}", body_str);

		Ok(body)
	}

	fn json_req<S: Serialize>(method: &str, params: S, id: u32) -> Value {
		json!({
			"method": method,
			"params": params,
			"jsonrpc": "2.0",
		})
	}

	pub fn hex_to_balance(hex: &str) -> u128 {
		// let hex: &str = "70ce0360818118000000000000000000";
		let big_endian_value: u128 = u128::from_str_radix(hex, 16).unwrap();
		// Convert the value to little endian
		// let little_endian_value = big_endian_value.to_le();
		let little_endian_value = big_endian_value.swap_bytes();
		little_endian_value
	}

	pub fn total_user_vault_collateral(token: &str) -> u128 {
		log::info!("calling total_user_vault_collateral");
		let module_name = "VaultRegistry";
		let storage_name = "TotalUserVaultCollateral";
		let storage_key = generate_storage_key(module_name, &storage_name);
		let mut storage_key_hash = to_hex(storage_key);

		if token.ne(&String::from("DOT")) {
			storage_key_hash = "0x".to_owned()
				+ &storage_key_hash
				+ "ed11b90b07067c86130c95aabfcb699c01020000000001";
		} else {
			storage_key_hash =
				"0x".to_owned() + &storage_key_hash + "d6bfa4fbbbb302d0f4e13a890467318100000001";
		}

		let result = fetch_data("state_getKeys", &storage_key_hash);
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

						locked = hex_to_balance(&stripped_string);

						log::info!("Result: issued {}", locked)
					},
					None => log::error!("Result is null"),
				}
			},
			Err(e) => {
				log::error!("HTTP error: ");
			},
		};
		locked
	}

	pub fn oracle(token: &str) -> u128 {
		log::info!("calling total_user_vault_collateral");
		let module_name = "Oracle";
		let storage_name = "Aggregate";
		let storage_key = generate_storage_key(module_name, &storage_name);
		let mut storage_key_hash = to_hex(storage_key);

		if token.ne(&String::from("DOT")) {
			storage_key_hash = "0x".to_owned()
				+ &storage_key_hash
				+ "e8ee4335018f6743c682ee73dfe0674c000102000000";
		} else {
			storage_key_hash =
				"0x".to_owned() + &storage_key_hash + "7b79be5e9b370ba6d080e4e2af7b7b89000000";
		}

		let result = fetch_data("state_getKeys", &storage_key_hash);
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

						locked = hex_to_balance(&stripped_string);

						log::info!("Result: issued {}", locked)
					},
					None => log::error!("Result is null"),
				}
			},
			Err(e) => {
				log::error!("HTTP error: ");
			},
		};
		locked
	}
}
