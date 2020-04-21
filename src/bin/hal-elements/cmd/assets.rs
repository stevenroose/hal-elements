use clap;

use cmd;

use std::io::Write;
use std::str::FromStr;

use bitcoin::hashes::hex::FromHex;
use bitcoin::hashes::{sha256, Hash};
use elements::{AssetId, ContractHash, OutPoint};

use hal_elements::assets::{AssetContract, AssetContractEntity, AssetContractInfo};

pub fn subcommand<'a>() -> clap::App<'a, 'a> {
	cmd::subcommand_group("assets", "work with assets")
		.subcommand(cmd_asset_id())
		.subcommand(cmd_contract())
}

pub fn execute<'a>(matches: &clap::ArgMatches<'a>) {
	match matches.subcommand() {
		("asset-id", Some(ref m)) => exec_asset_id(&m),
		("contract", Some(ref m)) => exec_contract(&m),
		(_, _) => unreachable!("clap prints help"),
	};
}

pub fn cmd_asset_id<'a>() -> clap::App<'a, 'a> {
	cmd::subcommand("asset-id", "calculate asset IDs")
		.long_about(
			r#"
Calculate an asset ID. Provide either of the following:

- the asset entropy hex
- the prevout and the contract hash
- the prevout and the raw JSON contract
"#,
		)
		.args(&[
			cmd::arg("entropy", "the hexadecimal asset entropy").long("entropy").required(false),
			cmd::arg("prevout", "the issuance tx prevout in hex").long("prevout").required(false),
			cmd::arg("contract-hash", "the issuance contract hash in hex")
				.long("contract-hash")
				.required(false),
			cmd::arg("contract-json", "the issuance contract JSON object")
				.long("contract-json")
				.required(false),
		])
}

pub fn exec_asset_id<'a>(matches: &clap::ArgMatches<'a>) {
	let entropy = if let Some(entropy) = matches.value_of("entropy") {
		// If the entropy is provided, nothing else should be.
		if matches.is_present("prevout")
			|| matches.is_present("contract-hash")
			|| matches.is_present("contract-json")
		{
			panic!("If --entropy is given, no other arguments should be provided");
		}
		sha256::Midstate::from_hex(entropy).expect("invalid entropy hex")
	} else {
		let prevout_str =
			matches.value_of("prevout").expect("Not enough information provided, use --help.");
		let prevout = OutPoint::from_str(prevout_str).expect("invalid prevout value");

		let contract_hash = if let Some(ch) = matches.value_of("contract-hash") {
			ContractHash::from_hex(ch).expect("invalid contract hash hex")
		} else if let Some(contract) = matches.value_of("contract-json") {
			let json: serde_json::Value = contract.parse().expect("invalid contract JSON");
			let mut engine = ContractHash::engine();
			serde_json::to_writer(&mut engine, &json).unwrap();
			ContractHash::from_engine(engine)
		} else {
			panic!("Not enough information provided, use --help.")
		};

		AssetId::generate_asset_entropy(prevout, contract_hash)
	};

	write!(::std::io::stdout(), "{}", AssetId::from_entropy(entropy)).unwrap()
}

pub fn cmd_contract<'a>() -> clap::App<'a, 'a> {
	cmd::subcommand("contract", "asset contracts for issuers")
		.long_about(
			r#"
Use this command to validate an existing JSON contract or create a new one.

The following JSON fields and arguments are supported:

- "entity": an object with the following values:
	- "domain"/--entity-domain: the DNS domain of the issuing entity
- "issuer_pubkey"/--issuer-pubkey: the public key authorized to change the asset contract
- "name"/--name: the name of the asset
- "precision"/--precision: the precision of the asset (i.e. the number of decimals of
	the smallest unit)
- "ticker"/--ticker: the ticker of the asset
- "version"/--version: the version of the contract format (should be 0)

Example of a contract JSON:
{"entity":{"domain":"tether.to"},"issuer_pubkey":"0337cceec0beea0232ebe14cba0197a9fbd45fcf2ec946749de920e71434c2b904","name":"Tether USD","precision":8,"ticker":"USDt","version":0}
"#,
		)
		.args(&[
			cmd::arg("json", "the JSON contract").long("json").required(false),
			cmd::arg("entity-domain", "the DNS domain of the issuing entity")
				.long("entity-domain").required(false),
			cmd::arg("issuer-pubkey", "the public key authorized to change the asset contract")
				.long("issuer-pubkey").required(false),
			cmd::arg("name", "the name of the asset").long("name").required(false),
			cmd::arg("precision", "the name of the asset").long("precision").required(false),
			cmd::arg("ticker", "the name of the asset").long("ticker").required(false),
			cmd::arg("version", "the version of the asset format (omit for default)")
				.long("version").required(false),
		])
}

pub fn exec_contract<'a>(matches: &clap::ArgMatches<'a>) {
	let (contract, raw) = if let Some(json) = matches.value_of("json") {
		// We parse and reserialize the JSON to remove whitespace but preserve the exact structure.
		let json: serde_json::Value = json.parse().expect("invalid contract JSON");
		let contract: AssetContract =
			serde_json::from_value(json.clone()).expect("invalid contract structure");
		for (key, _) in contract.additional_fields.iter() {
			eprintln!("Field \"{}\" in the contract is unknown.", key);
		}
		(contract, serde_json::to_string(&json).unwrap())
	} else {
		let contract = AssetContract {
			entity: matches.value_of("entity-domain").map(|domain| AssetContractEntity {
				domain: Some(domain.to_owned()),
			}),
			issuer_pubkey: matches
				.value_of("issuer-pubkey")
				.map(|k| k.parse().expect("invalid issuer pubkey")),
			name: matches.value_of("name").expect("no name provided").to_owned(),
			precision: matches
				.value_of("precision")
				.expect("no precision provided")
				.parse()
				.expect("invalid precision value"),
			ticker: matches.value_of("ticker").expect("no ticker provided").to_owned(),
			version: matches.value_of("version").unwrap_or("0").parse().expect("invalid version"),
			additional_fields: Default::default(),
		};
		let raw = serde_json::to_string(&contract).unwrap();
		(contract, raw)
	};

	if contract.issuer_pubkey.is_none() {
		eprintln!("It's probably a good idea to provide an issuer public key.");
	}
	if contract.entity.is_none() || contract.entity.as_ref().unwrap().domain.is_none() {
		eprintln!("It's probably a good idea to provide a domain name for the issuing entity.");
	}

	let info = AssetContractInfo {
		contract: contract,
		contract_hash: ContractHash::hash(&raw.as_bytes()),
		raw_contract: raw,
	};
	cmd::print_output(matches, &info)
}
