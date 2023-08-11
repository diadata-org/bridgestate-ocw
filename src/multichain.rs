use crate::{impls::RPCCalls, Asset, AssetCollector, MultichainData};
use codec::{Decode, Encode};
use frame_support::RuntimeDebug;
use scale_info::prelude::string::String;
use serde::{Deserialize, Serialize};
use sp_runtime::offchain::{http::Request, storage::StorageValueRef, Duration};
use sp_std::collections::btree_map::BTreeMap;
use sp_std::{boxed::Box, str, vec, vec::Vec};

const FETCH_TIMEOUT_PERIOD: u64 = 5_000; // in milli-seconds
const RPC_FETCH_TIMEOUT_PERIOD: u64 = 5_000; // in milli-seconds

const CHAINS_STORAGE_KEY: &[u8] = b"collateral-reader::multichain-chains-store";
const ASSETS_STORAGE_KEY: &[u8] = b"collateral-reader::multichain-assets-store";
const ASSOCIATED_ASSETS_STORAGE_KEY: &[u8] =
	b"collateral-reader::multichain-associated-assets-store";
const STATS_STORAGE_KEY: &[u8] = b"collateral-reader::multichain-stats-store";
const WORK_IN_PROGRESS_STORAGE_KEY: &[u8] = b"collateral-reader::multichain-work-in-progress";

type AssetId = Vec<u8>;
type ChainId = Vec<u8>;

const ETH_CHAIN_ID: &[u8] = b"1";

#[derive(Clone, Serialize, Deserialize, Encode, Decode, Default, RuntimeDebug)]
pub struct Chain {
	id: ChainId,
	name: Vec<u8>,
	rpc: Vec<u8>,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default, RuntimeDebug)]
#[serde(rename_all = "camelCase")]
struct MultichainTokenAssociatedAssetOnAnotherChain {
	address: String,
	router: String,
	chain_id: String,
	symbol: String,
	anytoken: Option<AnyToken>,
	fromanytoken: Option<AnyToken>,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default, RuntimeDebug, Clone)]
#[serde(rename_all = "camelCase")]
struct AnyToken {
	name: String,
	symbol: String,
	decimals: u64,
	address: String,
	chain_id: Option<String>,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default, RuntimeDebug)]
#[serde(rename_all = "camelCase")]
struct MultichainToken {
	chain_id: String,
	name: String,
	symbol: String,
	address: String,
	decimals: u64,
	dest_chains: BTreeMap<String, BTreeMap<String, MultichainTokenAssociatedAssetOnAnotherChain>>,
}

#[derive(Serialize, Deserialize, Encode, Decode, Default, RuntimeDebug)]
struct ChainResponse {
	name: String,
	rpc: String,
}

#[derive(Clone, Encode, Decode, Debug)]
struct AssociatedAsset {
	asset_id_on_eth_chain: AssetId,
	asset_id: AssetId,
	chain_id: ChainId,
	address: Vec<u8>,
	address_that_has_locked_assets: Vec<u8>,
	address_that_has_issued_assets: Vec<u8>,
	minted_asset: bool,
}

#[derive(Clone, Encode, Decode, Debug)]
struct MultichainAssetStats {
	asset_id: AssetId,
	locked: u128,
	issued: u128,
}

#[derive(Serialize, Deserialize, Clone)]
struct BalanceRpc {
	jsonrpc: String,
	method: String,
	params: (BalanceParams, String),
	id: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct BalanceRpcResponse {
	result: String,
}

#[derive(Serialize, Deserialize, Clone)]
struct BalanceParams {
	to: String,
	data: String,
}

fn crop_letters(s: &str, pos: usize) -> &str {
	match s.char_indices().skip(pos).next() {
		Some((pos, _)) => &s[pos..],
		None => "",
	}
}

#[derive(Debug, Clone)]
pub enum Error {
	HttpFetchingError,
	DeserializeToObjError,
	GetChainsError,
	GetAssetsError,
	GetAssetsStatsError,
}

pub fn get_assets_stats_job() {
	let work_in_progress_storage: StorageValueRef =
		StorageValueRef::persistent(WORK_IN_PROGRESS_STORAGE_KEY);
	let work_in_progress = work_in_progress_storage.get::<bool>().unwrap();
	if work_in_progress.is_none() || work_in_progress == Some(false) {
		work_in_progress_storage.set(&true);
		let result = get_assets_stats();
		if let Err(e) = result {
			log::info!("Error getting assets stats: {:?}", e);
		}
		work_in_progress_storage.set(&false);
	}
}

fn get_chains() -> Result<Vec<Chain>, Error> {
	let multichain_chains_store: StorageValueRef = StorageValueRef::persistent(CHAINS_STORAGE_KEY);
	if let Ok(Some(chains)) = multichain_chains_store.get::<Vec<Chain>>() {
		// chains has already been fetched. Return early.
		return Ok(chains);
	} else {
		const HTTP_REMOTE_REQUEST: &str = "https://scanapi.multichain.org/data/chain?type=mainnet";
		let resp_str = match call_api(HTTP_REMOTE_REQUEST) {
			Ok(res) => res,
			Err(err) => return Err(err),
		};
		// Deserializing JSON to struct, thanks to `serde` and `serde_derive`
		let response: BTreeMap<String, ChainResponse> =
			serde_json::from_slice(&resp_str).map_err(|e| {
				log::error!("{:?}", e);
				<Error>::DeserializeToObjError
			})?;
		let mut chains: Vec<Chain> = vec![];
		for (key, value) in response.iter() {
			chains.push(Chain {
				id: key.clone().into(),
				name: value.name.clone().into(),
				rpc: value.rpc.clone().into(),
			})
		}
		multichain_chains_store.set(&chains);
		Ok(chains)
	}
}

fn get_assets() -> Result<Vec<Asset>, Error> {
	let chains = get_chains();
	if let Err(_err) = chains {
		log::error!("Error getting chains.")
	}
	let multichain_assets_store: StorageValueRef = StorageValueRef::persistent(ASSETS_STORAGE_KEY);
	if let Ok(Some(assets)) = multichain_assets_store.get::<Vec<Asset>>() {
		// assets has already been fetched. Return early.
		return Ok(assets);
	} else {
		const HTTP_REMOTE_REQUEST: &str = "https://bridgeapi.multichain.org/v4/tokenlistv4/1";
		let resp_str = match call_api(HTTP_REMOTE_REQUEST) {
			Ok(res) => res,
			Err(err) => return Err(err),
		};
		// Deserializing JSON to struct, thanks to `serde` and `serde_derive`
		let response: BTreeMap<String, MultichainToken> = serde_json::from_slice(&resp_str)
			.map_err(|e| {
				log::error!("parse error {:?}", e);
				<Error>::DeserializeToObjError
			})?;
		let mut assets: Vec<Asset> = vec![];
		let mut associated_assets: Vec<AssociatedAsset> = vec![];
		let whitelisted_asset_symbols: Vec<&str> =
			vec!["ETH", "WETH", "WBTC", "USDC", "USDT", "DAI"];
		for (_, value) in response.iter() {
			let symbol = &value.symbol.as_str().clone();
			if !whitelisted_asset_symbols.contains(&symbol) {
				continue;
			}
			let asset = Asset {
				name: value.name.clone().into(),
				address: value.address.clone().into(),
				metadata: "".into(),
				chain: value.chain_id.clone().into(),
				decimals: value.decimals,
				symbol: value.symbol.clone().into(),
			};
			for (_, associated_asset_dest_chains) in value.dest_chains.iter() {
				for (_, associated_asset) in associated_asset_dest_chains.iter() {
					if associated_asset.anytoken.is_none()
						|| associated_asset.fromanytoken.is_none()
					{
						// skip this object if it doesn't have both anytoken and fromanytoken.
						continue;
					}
					let fromanytoken = associated_asset.fromanytoken.clone().unwrap();
					associated_assets.push(AssociatedAsset {
						asset_id_on_eth_chain: value.symbol.clone().into(),
						asset_id: associated_asset.symbol.clone().into(),
						chain_id: associated_asset.chain_id.clone().into(),
						address: associated_asset.address.clone().into(),
						address_that_has_locked_assets: associated_asset
							.anytoken
							.clone()
							.unwrap()
							.address
							.clone()
							.into(),
						address_that_has_issued_assets: fromanytoken.address.clone().into(),
						minted_asset: fromanytoken.chain_id
							== Some(String::from(str::from_utf8(ETH_CHAIN_ID).unwrap())),
					})
				}
			}
			assets.push(asset);
		}
		multichain_assets_store.set(&assets);

		let multichain_associated_assets_store: StorageValueRef =
			StorageValueRef::persistent(ASSOCIATED_ASSETS_STORAGE_KEY);
		multichain_associated_assets_store.set(&associated_assets);

		return Ok(assets);
	}
}

fn call_api(url: &str) -> Result<Vec<u8>, Error> {
	let request = Request::get(url);

	// Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
	let timeout = sp_io::offchain::timestamp().add(Duration::from_millis(FETCH_TIMEOUT_PERIOD));

	let pending = request
		.deadline(timeout) // Setting the timeout time
		.send() // Sending the request out by the host
		.map_err(|e| {
			log::error!("{:?}", e);
			<Error>::HttpFetchingError
		})?;

	// By default, the http request is async from the runtime perspective. So we are asking the
	//   runtime to wait here
	// The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
	//   ref: https://docs.substrate.io/rustdocs/latest/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
	let response = pending
		.try_wait(timeout)
		.map_err(|e| {
			log::error!("req error:: {:?}", e);
			<Error>::HttpFetchingError
		})?
		.map_err(|e| {
			log::error!("req error:: {:?}", e);
			<Error>::HttpFetchingError
		})?;

	if response.code != 200 {
		log::error!("Unexpected http request status code: {}", response.code);
		return Err(<Error>::HttpFetchingError);
	}

	// Next we fully read the response body and collect it to a vector of bytes.
	let resp_bytes = response.body().collect::<Vec<u8>>();
	Ok(resp_bytes)
}

fn get_erc20_token_balance(
	url: &str,
	method: String,
	params: (BalanceParams, String),
) -> Result<Vec<u8>, Error> {
	let request_body = BalanceRpc {
		jsonrpc: String::from("2.0"),
		method: method.into(),
		params: params.into(),
		id: String::from("1"),
	};
	let request_body_str =
		serde_json::to_string(&request_body).map_err(|_| <Error>::GetAssetsError)?;
	let request = Request::post(url, vec![request_body_str.clone()])
		.add_header("content-type", "application/json");

	// Keeping the offchain worker execution time reasonable, so limiting the call to be within 3s.
	let timeout = sp_io::offchain::timestamp().add(Duration::from_millis(RPC_FETCH_TIMEOUT_PERIOD));

	let pending = request
		.deadline(timeout)
		.body(vec![request_body_str.clone()])
		.send() // Sending the request out by the host
		.map_err(|e| {
			log::error!("rpc request err: {:?}", e);
			<Error>::HttpFetchingError
		})?;

	// By default, the http request is async from the runtime perspective. So we are asking the
	//   runtime to wait here
	// The returning value here is a `Result` of `Result`, so we are unwrapping it twice by two `?`
	//   ref: https://docs.substrate.io/rustdocs/latest/sp_runtime/offchain/http/struct.PendingRequest.html#method.try_wait
	let response = pending
		.try_wait(timeout)
		.map_err(|e| {
			log::error!("rpc error: {:?}", e);
			<Error>::HttpFetchingError
		})?
		.map_err(|e| {
			log::error!("rpc error: {:?}", e);
			<Error>::HttpFetchingError
		})?;

	if response.code != 200 {
		log::error!("Unexpected http request status code: {}", response.code);
		return Err(<Error>::HttpFetchingError);
	}

	// Next we fully read the response body and collect it to a vector of bytes.
	let resp_bytes = response.body().collect::<Vec<u8>>();
	Ok(resp_bytes)
}

fn get_assets_stats() -> Result<(), Error> {
	let multichain_assets_store: StorageValueRef = StorageValueRef::persistent(ASSETS_STORAGE_KEY);
	let assets = multichain_assets_store.get::<Vec<Asset>>().map_err(|e| {
		log::error!("multichain_assets_store {:?}", e);
		<Error>::GetAssetsStatsError
	})?;
	if assets.is_none() {
		// early return for when assets are not ready yet.
		return Ok(());
	}
	let multichain_associated_assets_store: StorageValueRef =
		StorageValueRef::persistent(ASSOCIATED_ASSETS_STORAGE_KEY);
	let multichain_chains_store: StorageValueRef = StorageValueRef::persistent(CHAINS_STORAGE_KEY);
	let chains = multichain_chains_store.get::<Vec<Chain>>().map_err(|e| {
		log::error!("chains {:?}", e);
		<Error>::GetAssetsStatsError
	})?;
	if chains.is_none() {
		// early return for when chains are not ready yet.
		return Ok(());
	}
	let mut all_stats: Vec<MultichainAssetStats> = vec![];
	let unwrapped_chains = chains.unwrap();
	let associated_assets =
		multichain_associated_assets_store.get::<Vec<AssociatedAsset>>().map_err(|e| {
			log::error!("multichain_associated_assets_store {:?}", e);
			<Error>::GetAssetsStatsError
		})?;
	let multichain_stats_store: StorageValueRef = StorageValueRef::persistent(STATS_STORAGE_KEY);
	let prev_stats = multichain_stats_store.get::<Vec<MultichainAssetStats>>().map_err(|e| {
		log::error!("multichain_stats_store {:?}", e);
		<Error>::GetAssetsStatsError
	})?;
	for asset in assets.unwrap().iter() {
		let associated_assets_of_current_asset: Vec<AssociatedAsset> = associated_assets
			.clone()
			.unwrap()
			.iter()
			.filter(|i| i.asset_id_on_eth_chain == asset.symbol)
			.cloned()
			.collect();
		let mut locked_addresses_map: BTreeMap<Vec<u8>, (Vec<u8>, AssetId)> = BTreeMap::new();
		let mut issued_addresses_map: BTreeMap<Vec<u8>, (Vec<u8>, AssetId)> = BTreeMap::new();
		for associated_asset in associated_assets_of_current_asset.iter() {
			let chain = unwrapped_chains.iter().find(|x| x.id == associated_asset.chain_id);
			if chain.is_none() {
				continue;
			}
			let chain_rpc_url = chain.unwrap().rpc.clone();
			let eth_chain =
				unwrapped_chains.iter().find(|x| x.id == ETH_CHAIN_ID.to_vec()).unwrap();
			let eth_chain_rpc_url = eth_chain.rpc.clone();
			locked_addresses_map.insert(
				associated_asset.address_that_has_locked_assets.clone(),
				(chain_rpc_url, associated_asset.address.clone()),
			);
			issued_addresses_map.insert(
				associated_asset.address_that_has_issued_assets.clone(),
				(eth_chain_rpc_url, asset.address.clone()),
			);
		}
		let total_issued_amount = get_total_amount(issued_addresses_map);
		let total_locked_amount = get_total_amount(locked_addresses_map);
		let tla = total_locked_amount.clone();
		let tia = total_issued_amount.clone();
		if tla > 0 && tia > 0 {
			all_stats.push(MultichainAssetStats {
				asset_id: asset.symbol.clone(),
				locked: total_locked_amount,
				issued: total_issued_amount,
			});
		} else if !prev_stats.is_none() {
			let unwrapped_prev_stats = prev_stats.clone().unwrap();
			let asset_prev_stats =
				unwrapped_prev_stats.iter().find(|i| i.asset_id == asset.symbol.clone());
			if asset_prev_stats.is_some() {
				all_stats.push(asset_prev_stats.unwrap().clone());
			}
		}
	}
	log::info!("all_stats: {:?}", all_stats);
	let multichain_stats_store: StorageValueRef = StorageValueRef::persistent(STATS_STORAGE_KEY);
	multichain_stats_store.set(&all_stats);
	Ok(())
}

fn get_total_amount(address_map: BTreeMap<Vec<u8>, (Vec<u8>, AssetId)>) -> u128 {
	let mut total_amount: u128 = 0;
	for (router_address, (chain_rpc_url, asset_address)) in address_map.iter() {
		let address = str::from_utf8(&router_address).unwrap();
		let balance_params = BalanceParams {
			to: String::from_utf8(asset_address.clone()).unwrap(),
			// `0x70a08231` is the `balanceOf` hash function selector
			data: alloc::format!("0x70a08231000000000000000000000000{}", crop_letters(address, 2)),
		};
		let result = get_erc20_token_balance(
			str::from_utf8(&chain_rpc_url).unwrap(),
			String::from("eth_call"),
			(balance_params, String::from("latest")),
		)
		.map_err(|_e| <Error>::GetAssetsStatsError);
		match result {
			Ok(res) => {
				let parsed = serde_json::from_slice(&res).map_err(|e| {
					log::info!("get balance deserialize error: {:?}", e);
					<Error>::DeserializeToObjError
				});
				if let Err(_e) = parsed {
					continue;
				}
				let amount: BalanceRpcResponse = parsed.unwrap();
				let parsed_amount =
					u128::from_str_radix(crop_letters(&amount.result, 2), 16).unwrap();
				total_amount += parsed_amount;
			},
			Err(_e) => {
				continue;
			},
		}
	}
	total_amount
}

pub struct MultichainRPCHelper {}

impl RPCCalls for MultichainRPCHelper {
	fn get_supported_assets(&self) -> Result<Vec<Asset>, &'static str> {
		let assets = get_assets();
		if let Err(_e) = assets {
			return Err("MultichainRPC, error getting supported assets.");
		}
		Ok(assets.unwrap())
	}

	fn get_locked(&self, asset: Vec<u8>) -> Result<u128, &'static str> {
		get_assets_stats_job();
		let multichain_stats_store: StorageValueRef =
			StorageValueRef::persistent(STATS_STORAGE_KEY);
		let stats = multichain_stats_store
			.get::<Vec<MultichainAssetStats>>()
			.map_err(|e| {
				log::error!("multichain_stats_store {:?}", e);
				return b"MultichainRPCHelper, error getting asset stats.";
			})
			.unwrap();
		if stats.is_none() {
			return Ok(0);
		}
		let long_lived_stats = stats.unwrap().clone();
		let asset_stats = long_lived_stats.iter().find(|i| i.asset_id == asset);
		if asset_stats.is_none() {
			return Err("MultichainRPCHelper, error getting issued amount.");
		}
		Ok(asset_stats.unwrap().locked)
	}

	fn get_issued(&self, asset: Vec<u8>) -> Result<u128, &'static str> {
		get_assets_stats_job();
		let multichain_stats_store: StorageValueRef =
			StorageValueRef::persistent(STATS_STORAGE_KEY);
		let stats = multichain_stats_store
			.get::<Vec<MultichainAssetStats>>()
			.map_err(|e| {
				log::error!("multichain_stats_store {:?}", e);
				return b"MultichainRPCHelper, error getting asset stats.";
			})
			.unwrap();
		if stats.is_none() {
			return Ok(0);
		}
		let long_lived_stats = stats.unwrap().clone();
		let asset_stats = long_lived_stats.iter().find(|i| i.asset_id == asset);
		if asset_stats.is_none() {
			return Err("MultichainRPCHelper, error getting issued amount.");
		}
		Ok(asset_stats.unwrap().issued)
	}

	fn get_minted_asset(&self, asset: Vec<u8>) -> Result<Vec<u8>, &'static str> {
		let multichain_associated_assets_store: StorageValueRef =
			StorageValueRef::persistent(ASSOCIATED_ASSETS_STORAGE_KEY);
		let associated_assets =
			multichain_associated_assets_store.get::<Vec<AssociatedAsset>>().map_err(|e| {
				log::error!("multichain_associated_assets_store {:?}", e);
				return "MultichainRPCHelper, error getting assets.";
			});
		if let Err(_x) = associated_assets {
			return Ok("".into());
		}
		let unwrapped_associated_assets = associated_assets.unwrap();
		if unwrapped_associated_assets.is_none() {
			return Ok("".into());
		}
		let long_lived_associated_assets = unwrapped_associated_assets.clone().unwrap();
		let associated_asset = long_lived_associated_assets
			.iter()
			.find(|i| i.asset_id_on_eth_chain == asset && i.minted_asset == true)
			.clone();
		if associated_asset.is_none() {
			return Err("MultichainRPCHelper, error getting minted asset.");
		}
		Ok(associated_asset.unwrap().asset_id.clone())
	}

	fn get_associated_assets(&self, minted_asset: Vec<u8>) -> Result<Vec<u8>, &'static str> {
		let multichain_associated_assets_store: StorageValueRef =
			StorageValueRef::persistent(ASSOCIATED_ASSETS_STORAGE_KEY);
		let all_associated_assets = multichain_associated_assets_store
			.get::<Vec<AssociatedAsset>>()
			.map_err(|e| {
				log::error!("multichain_associated_assets_store {:?}", e);
				return b"MultichainRPCHelper, error getting assets.";
			})
			.unwrap()
			.unwrap();
		let associated_asset = all_associated_assets.iter().find(|i| i.asset_id == minted_asset);
		Ok(associated_asset.unwrap().asset_id_on_eth_chain.clone())
	}
}

impl AssetCollector for MultichainData {
	fn get_supported_assets(&self) -> Vec<Asset> {
		let helpers: Vec<Box<dyn RPCCalls>> = vec![Box::new(MultichainRPCHelper {})];

		for helper in helpers {
			let result = helper.get_supported_assets();
			match result {
				Ok(assets) => return assets,
				Err(e) => {
					log::info!("Error getting assets, {:?}", e);
					// "Error occurred, retrying with the next helper..."
					continue;
				},
			}
		}
		vec![]
	}

	fn get_locked(self, asset: Vec<u8>) -> u128 {
		let helpers: Vec<Box<dyn RPCCalls>> = vec![Box::new(MultichainRPCHelper {})];

		for helper in helpers {
			let result = helper.get_locked(asset.clone());
			match result {
				Ok(locked) => return locked,
				Err(e) => {
					// "Error occurred, retrying with the next helper..."
					log::info!("Error getting get_locked, {:?}", e);
					continue;
				},
			}
		}
		0
	}
	fn get_issued(self, asset: Vec<u8>) -> u128 {
		let helpers: Vec<Box<dyn RPCCalls>> = vec![Box::new(MultichainRPCHelper {})];

		for helper in helpers {
			let result = helper.get_issued(asset.clone());
			match result {
				Ok(issued) => return issued,
				Err(e) => {
					log::info!("Error getting get_issued, {:?}", e);
					// "Error occurred, retrying with the next helper..."
					continue;
				},
			}
		}
		0
	}
	fn get_minted_asset(self, asset: Vec<u8>) -> Vec<u8> {
		let helpers: Vec<Box<dyn RPCCalls>> = vec![Box::new(MultichainRPCHelper {})];

		for helper in helpers {
			let result = helper.get_minted_asset(asset.clone());
			match result {
				Ok(mintedasset) => return mintedasset,
				Err(e) => {
					log::info!("Error getting get_minted_asset, {:?}", e);
					// "Error occurred, retrying with the next helper..."
					continue;
				},
			}
		}
		vec![0]
	}

	fn get_associated_assets(self, minted_asset: Vec<u8>) -> Vec<u8> {
		let helpers: Vec<Box<dyn RPCCalls>> = vec![Box::new(MultichainRPCHelper {})];

		for helper in helpers {
			let result = helper.get_associated_assets(minted_asset.clone());
			match result {
				Ok(assets) => return assets,
				Err(e) => {
					// "Error occurred, retrying with the next helper..."
					log::info!("Error getting get_associated_assets, {:?}", e);
					continue;
				},
			}
		}
		vec![0]
	}
}
