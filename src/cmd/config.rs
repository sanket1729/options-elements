//! Create a call option on Elements
use bitcoin;
use elements::AssetId;

use cmd;
use serde::{Deserialize, Serialize};
use std::{fs::File, str::FromStr};

pub const CTRL_SK: &str = "cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy";

pub const CTRL_PK: &str = "039b6347398505f5ec93826dc61c19f47c66c0283ee9be980e29ce325a0f4679ef";

pub const BTC_ASSET: &str = "b2e15d0d7a0c94e4e2ce0fe6e8691b9e451377f6e46e8045a86f7c4b5d4f0f23";

pub fn cmd_create<'a>() -> clap::App<'a, 'a> {
    cmd::subcommand("init", "Initialize the options contract")
        .args(&cmd::opts_networks())
        .args(&[
            cmd::opt_yaml(),
            cmd::opt("lock-asset", "The asset that is locked in options contract")
                .takes_value(true)
                .required(true),
            cmd::opt(
                "claim-asset",
                "The asset that is spent to exercise the option",
            )
            .takes_value(true)
            .required(true),
            cmd::opt("opt-token", "The option token")
                .takes_value(true)
                .required(true),
            cmd::opt(
                "bene-token",
                "The beneficiary token(writer token) of the option",
            )
            .takes_value(true)
            .required(true),
            cmd::opt(
                "locked-asset-amount",
                "The amount in sat of the locked asset(default 10^8)",
            )
            .takes_value(true)
            .required(false)
            .default_value("100000000"),
            cmd::opt("btc-asset", "The btc asset id(default regtest btc id)")
                .takes_value(true)
                .required(false)
                .default_value(BTC_ASSET),
            cmd::opt(
                "out-file",
                "Path where to save the config file. Default current directory",
            )
            .takes_value(true)
            .required(false)
            .default_value("./opt_cfg.conf"),
        ])
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub lock_asset: AssetId,
    pub claim_asset: AssetId,
    pub opt_token: AssetId,
    pub bene_token: AssetId,
    pub locked_asset_amount: u64,
    pub control_pk: bitcoin::PublicKey,
    pub control_sk: bitcoin::PrivateKey,
    pub btc_asset: AssetId,
}

pub fn exec_init<'a>(matches: &clap::ArgMatches<'a>) {
    let lock_asset = matches
        .value_of("lock-asset")
        .expect("Lock Asset missing")
        .parse::<elements::AssetId>()
        .expect("Invalid Lock Asset");

    let claim_asset = matches
        .value_of("claim-asset")
        .expect("claim Asset missing")
        .parse::<elements::AssetId>()
        .expect("Invalid claim Asset");

    let opt_token = matches
        .value_of("opt-token")
        .expect("Opt token missing")
        .parse::<elements::AssetId>()
        .expect("Invalid opt token AssetId");

    let bene_token = matches
        .value_of("bene-token")
        .expect("bene token missing")
        .parse::<elements::AssetId>()
        .expect("Invalid Bene token assetId");

    let locked_asset_amount = matches
        .value_of("locked-asset-amount")
        .expect("Locked asset amount error")
        .parse::<u64>()
        .expect("Invalid claim Asset amount");

    let btc_asset = matches
        .value_of("btc-asset")
        .expect("btc asset missing")
        .parse::<elements::AssetId>()
        .expect("Invalid btc asset assetId");

    let control_sk = bitcoin::PrivateKey::from_wif(CTRL_SK).unwrap();
    let control_pk = bitcoin::PublicKey::from_str(CTRL_PK).unwrap();

    let cfg = Config {
        lock_asset,
        claim_asset,
        opt_token,
        bene_token,
        locked_asset_amount,
        control_pk,
        control_sk,
        btc_asset,
    };
    let out_path = matches.value_of("out-file").expect("path");

    let file = File::create(&out_path).expect("failed to PSBT file for writing");
    serde_yaml::to_writer(file, &cfg).expect("Writing error")
}
