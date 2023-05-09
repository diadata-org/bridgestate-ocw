use crate::AssetCollector;
use crate::AssetData;
use crate::Asset;

use sp_std::{convert::TryInto, str, vec, vec::Vec};


impl AssetCollector for AssetData {
    fn get_supported_assets(&self) -> Vec<Asset> {
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
        assets.push(Asset {
            address: b"2".to_vec(),
            chain: b"interlay".to_vec(),
            metadata: b"".to_vec(),
            decimals: 0,
            symbol: b"IBTC".to_vec(),
            name: b"IBTC".to_vec(),
            // storage_key:
            // b"0x99971b5749ac43e0235e41b0d378691857c875e4cff74148e4628f264b974c80d67c5ba80ba065480001".to_vec()
        });
        assets
    }

    fn get_locked(asset: Vec<u8>) -> u64 {
        45
    }
    fn get_issued(asset: Vec<u8>) -> u64 {
        44
    }
    fn get_minted_asset(asset: Vec<u8>) -> Vec<u8> {
        b"LDOT".to_vec()
    }

    fn get_associated_assets(minted_asset: Vec<u8>) -> Vec<u8> {
        b"DOT".to_vec()
    }
}
