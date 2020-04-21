use std::collections::HashMap;

use bitcoin::PublicKey;
use elements::ContractHash;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct AssetContractEntity {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub domain: Option<String>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct AssetContract {
	#[serde(skip_serializing_if = "Option::is_none")]
	pub entity: Option<AssetContractEntity>,
	#[serde(skip_serializing_if = "Option::is_none")]
	pub issuer_pubkey: Option<PublicKey>,
	pub name: String,
	pub precision: u8,
	pub ticker: String,
	pub version: usize,

    #[serde(flatten)]
    pub additional_fields: HashMap<String, serde_json::Value>,
}

#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
pub struct AssetContractInfo {
	pub contract: AssetContract,
	pub raw_contract: String,
	pub contract_hash: ContractHash,
}
