
pub mod Keys {
    use sp_core::storage::StorageKey;
    use sp_io::hashing::{blake2_256, twox_128};
    use sp_std::vec::Vec;
    use scale_info::prelude::string::String;
	/// create key for a simple value.
	pub fn value(module: &[u8], storage: &[u8]) -> StorageKey {
		let mut final_key = [0u8; 32];
		final_key[0..16].copy_from_slice(&twox_128(module ));
		final_key[16..32].copy_from_slice(&twox_128(storage ));
		StorageKey(final_key.to_vec())
	}

	/// create key for a map.
	pub fn map(module: String, storage: String, encoded_key: &[u8]) -> StorageKey {
		let module_key = twox_128(module.as_bytes());
		let storage_key = twox_128(storage.as_bytes());
		let key = blake2_256(encoded_key);
		let mut final_key = Vec::with_capacity(module_key.len() + storage_key.len() + key.len());
		final_key.extend_from_slice(&module_key);
		final_key.extend_from_slice(&storage_key);
		final_key.extend_from_slice(&key);
		StorageKey(final_key)
	}
/* 
	/// create key for a linked_map head.
	pub fn linked_map_head(module: String, storage: String) -> StorageKey {
		let head_prefix = "HeadOf".to_string() + &storage;
		let mut final_key = [0u8; 32];
		final_key[0..16].copy_from_slice(&twox_128(module.as_bytes()));
		final_key[16..32].copy_from_slice(&twox_128(head_prefix.as_bytes()));
		StorageKey(final_key.to_vec())
	}
    */
}