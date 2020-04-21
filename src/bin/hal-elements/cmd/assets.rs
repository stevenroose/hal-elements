use clap;

use cmd;

use std::io::Write;
use std::str::FromStr;

use bitcoin::hashes::hex::FromHex;
use bitcoin::hashes::{sha256, Hash};
use elements::{AssetId, ContractHash, OutPoint};

pub fn subcommand<'a>() -> clap::App<'a, 'a> {
	cmd::subcommand_group("assets", "work with assets").subcommand(cmd_asset_id())
}

pub fn execute<'a>(matches: &clap::ArgMatches<'a>) {
	match matches.subcommand() {
		("asset-id", Some(ref m)) => exec_asset_id(&m),
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
