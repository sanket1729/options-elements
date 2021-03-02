//! Create a call option on Elements
use bitcoin::{Amount, PrivateKey, PublicKey};
use elements::{
    encode::serialize, encode::serialize_hex, secp256k1_zkp, Address, AssetId, AssetIssuance,
    OutPoint, SigHashType, Transaction, TxIn, TxInWitness,
};

use chrono::{Date, NaiveDate, NaiveTime, Utc};
use elements::hashes::hex::FromHex;
use elements::{confidential, AddressParams, Script, TxOut};
use miniscript::{descriptor::CovenantDescriptor, Segwitv0};
use miniscript::{Descriptor, DescriptorTrait, Miniscript};
// use elements::pset::PartiallySignedTransaction as Pset;

use cmd;
use std::{error, fmt, fs::File, str::FromStr};

use crate::cmd::Config;

pub const CTRL_SK: &str = "cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy";

pub const CTRL_PK: &str = "039b6347398505f5ec93826dc61c19f47c66c0283ee9be980e29ce325a0f4679ef";

pub const BTC_ASSET: [u8; 32] = [
    0x23, 0x0f, 0x4f, 0x5d, 0x4b, 0x7c, 0x6f, 0xa8, 0x45, 0x80, 0x6e, 0xe4, 0xf6, 0x77, 0x13, 0x45,
    0x9e, 0x1b, 0x69, 0xe8, 0xe6, 0x0f, 0xce, 0xe2, 0xe4, 0x94, 0x0c, 0x7a, 0x0d, 0x5d, 0xe1, 0xb2,
];

pub fn subcommand<'a>() -> clap::App<'a, 'a> {
    cmd::subcommand_group("call", "Call options functions")
        .subcommand(cmd_create())
        .subcommand(cmd_exercise())
        .subcommand(cmd_cancel())
        .subcommand(cmd_expiry())
        .subcommand(cmd_addcontract())
        .subcommand(cmd_finalize())
        .subcommand(cmd_claim_bene())
}

fn cmd_create<'a>() -> clap::App<'a, 'a> {
    cmd::subcommand("create", "Create a bitcoin call option")
        .args(&cmd::opts_networks())
        .args(&[
            cmd::opt_yaml(),
            cmd::opt("expiry", "The expiry of the option")
                .takes_value(true)
                .required(true),
            cmd::opt("strike", "The strike price of bitcoin")
                .takes_value(true)
                .required(true),
            cmd::opt(
                "cfg-file",
                "Path for reading config file. Default=./opt_cfg.conf",
            )
            .takes_value(true)
            .required(false)
            .default_value("./opt_cfg.conf"),
        ])
}

fn cmd_exercise<'a>() -> clap::App<'a, 'a> {
    cmd::subcommand(
        "exercise",
        "Exercise a call option. Outputs a raw transaction
        hex that should be provided to elements fundrawtransaction.
        Elements wallet does not understand covenants so it cannot estimate fees
        for the output. Set a higher fee manually to while calling fundrawtransction\n
        Example usage: elements-cli fundrawtransaction <tx_hex> '''{\"feeRate\": 0.03}'''",
    )
    .args(&cmd::opts_networks())
    .args(&[
        cmd::opt_yaml(),
        cmd::opt("expiry", "The expiry of the option")
            .takes_value(true)
            .required(true),
        cmd::opt("strike", "The strike price of bitcoin")
            .takes_value(true)
            .required(true),
        cmd::opt(
            "cfg-file",
            "Path for reading config file. Default=./opt_cfg.conf",
        )
        .takes_value(true)
        .required(false)
        .default_value("./opt_cfg.conf"),
    ])
}

fn cmd_cancel<'a>() -> clap::App<'a, 'a> {
    cmd::subcommand(
        "cancel",
        "Cancel a call option. Outputs a raw transaction
        hex that should be provided to elements fundrawtransaction.
        Elements wallet does not understand covenants so it cannot estimate fees
        for the output. Set a higher fee manually to while calling fundrawtransction\n
        Example usage: elements-cli fundrawtransaction <tx_hex> '''{\"feeRate\": 0.03}'''",
    )
    .args(&cmd::opts_networks())
    .args(&[
        cmd::opt_yaml(),
        cmd::opt("expiry", "The expiry of the option")
            .takes_value(true)
            .required(true),
        cmd::opt("strike", "The strike price of bitcoin")
            .takes_value(true)
            .required(true),
        cmd::opt(
            "cfg-file",
            "Path for reading config file. Default=./opt_cfg.conf",
        )
        .takes_value(true)
        .required(false)
        .default_value("./opt_cfg.conf"),
    ])
}

fn cmd_expiry<'a>() -> clap::App<'a, 'a> {
    cmd::subcommand(
        "expiry",
        "Claim a call option that has been expired. Outputs a raw transaction
        hex that should be provided to elements fundrawtransaction.\
        Elements wallet does not understand covenants so it cannot estimate fees
        for the output. Set a higher fee manually to while calling fundrawtransction\n
        Example usage: elements-cli fundrawtransaction <tx_hex> '''{\"feeRate\": 0.03}'''",
    )
    .args(&cmd::opts_networks())
    .args(&[
        cmd::opt_yaml(),
        cmd::opt("expiry", "The expiry of the option")
            .takes_value(true)
            .required(true),
        cmd::opt("strike", "The strike price of bitcoin")
            .takes_value(true)
            .required(true),
        cmd::opt(
            "cfg-file",
            "Path for reading config file. Default=./opt_cfg.conf",
        )
        .takes_value(true)
        .required(false)
        .default_value("./opt_cfg.conf"),
    ])
}

fn cmd_addcontract<'a>() -> clap::App<'a, 'a> {
    cmd::subcommand(
        "addcontract",
        "Step 2 for creating a transaction a call option. Provide a funded transaction created by
        calling fundrawtransaction on output transaction of exercise/expiry/cancel step.
        Provide covenant contract prevout and destination address for receiving locked asset.
        Incase of claimbene, the prevout is *NOT* the contract prevout, but the prevout created
        after contract is exercised.",
    )
    .args(&cmd::opts_networks())
    .args(&[
        cmd::opt_yaml(),
        cmd::opt("addr", "The address at which to receive locked asset")
            .takes_value(true)
            .required(true),
        cmd::opt("prev-txid", "The txid of contract")
            .takes_value(true)
            .required(true),
        cmd::opt("prev-vout", "The out index of contract txid")
            .takes_value(true)
            .required(true),
        cmd::opt("funded-tx", "The funded transaction from elements")
            .takes_value(true)
            .required(true),
        cmd::opt("expiry", "The expiry of the option")
            .takes_value(true)
            .required(true),
        cmd::opt("strike", "The strike price of bitcoin")
            .takes_value(true)
            .required(true),
        cmd::opt(
            "type",
            "The type of asset with contract operation.\
                Use either exercise/expiry/cancel/claimbene",
        )
        .takes_value(true)
        .required(true),
        cmd::opt(
            "cfg-file",
            "Path for reading config file. Default=./opt_cfg.conf",
        )
        .takes_value(true)
        .required(false)
        .default_value("./opt_cfg.conf"),
    ])
}

fn cmd_finalize<'a>() -> clap::App<'a, 'a> {
    cmd::subcommand(
        "finalize",
        "Final step for creating a transaction when dealing with call option.
        Sends the locked asset to the address supplied in add contract.
        Broadcast the output hex of this step to the network.",
    )
    .args(&cmd::opts_networks())
    .args(&[
        cmd::opt_yaml(),
        cmd::opt("signed-tx", "The funded transaction from elements")
            .takes_value(true)
            .required(true),
        cmd::opt("expiry", "The expiry of the option")
            .takes_value(true)
            .required(true),
        cmd::opt("strike", "The strike price of bitcoin")
            .takes_value(true)
            .required(true),
        cmd::opt(
            "type",
            "The type of asset with contract operation.\
                    Use either exercise/expiry/cancel/claimbene",
        )
        .takes_value(true)
        .required(true),
        cmd::opt(
            "cfg-file",
            "Path for reading config file. Default=./opt_cfg.conf",
        )
        .takes_value(true)
        .required(false)
        .default_value("./opt_cfg.conf"),
    ])
}

fn cmd_claim_bene<'a>() -> clap::App<'a, 'a> {
    cmd::subcommand("claimbene", "Claim an associated asset with the bene token")
        .args(&cmd::opts_networks())
        .args(&[
            cmd::opt_yaml(),
            cmd::opt("expiry", "The expiry of the option")
                .takes_value(true)
                .required(true),
            cmd::opt("strike", "The strike price of bitcoin")
                .takes_value(true)
                .required(true),
            cmd::opt(
                "cfg-file",
                "Path for reading config file. Default=./opt_cfg.conf",
            )
            .takes_value(true)
            .required(false)
            .default_value("./opt_cfg.conf"),
        ])
}

pub fn execute<'a>(matches: &clap::ArgMatches<'a>) {
    match matches.subcommand() {
        ("create", Some(ref m)) => exec_create(m),
        ("exercise", Some(ref m)) => exec_exercise(m),
        ("expiry", Some(ref m)) => exec_expiry(m),
        ("cancel", Some(ref m)) => exec_cancel(m),
        ("addcontract", Some(ref m)) => exec_addcontract(m),
        ("finalize", Some(ref m)) => exec_finalize(m),
        ("claimbene", Some(ref m)) => exec_claim_bene(m),
        (_, _) => unreachable!("clap prints help"),
    };
}

fn exec_exercise(matches: &clap::ArgMatches) {
    let contract = OptionContract::from_config(matches);

    let tx = contract.exercise_opt();
    println!("Raw tx: Pass this raw tx to fundrawtransction");
    println!("fundrawtransction <hex> '''{{\"feeRate\": 0.03}}'''");
    println!("{}", serialize_hex(&tx))
}

fn exec_cancel(matches: &clap::ArgMatches) {
    let contract = OptionContract::from_config(matches);

    let tx = contract.cancel_opt();
    println!("Raw tx: Pass this raw tx to fundrawtransction");
    println!("fundrawtransction <hex> '''{{\"feeRate\": 0.03}}'''");
    println!("{}", serialize_hex(&tx))
}

fn exec_expiry(matches: &clap::ArgMatches) {
    let contract = OptionContract::from_config(matches);

    let tx = contract.claim_expiry();
    println!("Raw tx: Pass this raw tx to fundrawtransction");
    println!("fundrawtransction <hex> '''{{\"feeRate\": 0.03}}'''");
    println!("{}", serialize_hex(&tx))
}

fn exec_claim_bene(matches: &clap::ArgMatches) {
    let contract = OptionContract::from_config(matches);

    let tx = contract.claim_bene();
    println!("Raw tx: Pass this raw tx to fundrawtransction");
    println!("fundrawtransction <hex> '''{{\"feeRate\": 0.03}}'''");
    println!("{}", serialize_hex(&tx))
}

fn exec_addcontract(matches: &clap::ArgMatches) {
    let contract = OptionContract::from_config(matches);

    let tx = matches.value_of("funded-tx").expect("Funded tx missing");
    let mut tx = elements::encode::deserialize(&Vec::<u8>::from_hex(tx).unwrap()).unwrap();

    let txid = matches
        .value_of("prev-txid")
        .expect("Prev txid missing")
        .parse::<elements::Txid>()
        .expect("Invalid prev txid");

    let vout = matches
        .value_of("prev-vout")
        .expect("vout of prev contract txid missing")
        .parse::<u32>()
        .expect("Invalid vout");

    let addr = matches
        .value_of("addr")
        .expect("Receiver Address missing")
        .parse::<elements::Address>()
        .expect("Malformed address");

    let ty = matches.value_of("type").expect("Contract type missing");

    if ty == "cancel" {
        contract.cancel_tx2(&mut tx, OutPoint::new(txid, vout), addr)
    } else if ty == "expiry" {
        contract.claim_expiry_tx2(&mut tx, OutPoint::new(txid, vout), addr)
    } else if ty == "exercise" {
        contract.exercise_opt_tx2(&mut tx, OutPoint::new(txid, vout), addr);
    } else if ty == "claimbene" {
        contract.claim_bene_tx2(&mut tx, OutPoint::new(txid, vout), addr);
    } else {
        panic!("Type must be expiry/exercise/cancel/claimbene")
    };

    println!("\n\n tx hex: \n\n");
    println!("{}", serialize_hex(&tx));
    let mut s = String::from("'''[");

    // In claim bene token the manually added input is of claim asset
    // while it is of locked asset in all other cases.
    let aux_gen = if ty == "claimbene" {
        confidential::Asset::Explicit(contract.claim_asset_params.asset)
    } else {
        confidential::Asset::Explicit(contract.locked_asset_params.asset)
    };
    // TODO: create a global context with static lifetime.
    let secp = secp256k1_zkp::Secp256k1::signing_only();
    let gen = aux_gen.into_asset_gen(&secp).unwrap();
    println!("Asset commitment list: Pass this as third arg to blindrawtransaction");
    for _ in 0..(tx.input.len() - 1) {
        // BUG: see https://github.com/ElementsProject/elements/issues/999
        s = format!("{}\"{}\", ", s, gen);
    }
    s = format!("{}\"{}\"]'''", s, gen);
    println!("{}", s);
    println!(
        "signrawtransction will be incomplete because we will sign the covenant input later step"
    );
    println!("Blind and sign the inputs that elements-cli can sign");
    println!("elements-cli blindrawtransaction <hex> true <asset_commitment_list>");
    println!("elements-cli signrawtransactionwithwallet <hex>");
}

fn exec_finalize(matches: &clap::ArgMatches) {
    let contract = OptionContract::from_config(matches);

    let tx = matches.value_of("signed-tx").expect("Signed tx missing");
    let mut tx = elements::encode::deserialize(&Vec::<u8>::from_hex(tx).unwrap()).unwrap();
    let ty = matches.value_of("type").expect("Contract type missing");

    contract.finalize_tx(&mut tx, ty);
    println!("{}", serialize_hex(&tx));
    println!("elements-cli sendrawtransaction <hex>");
}

fn exec_create<'a>(matches: &clap::ArgMatches<'a>) {
    let network = cmd::network(matches);

    let contract = OptionContract::from_config(matches);
    let addr = contract
        .deposit_addr(network.address_params())
        .expect("Contract Creation Error");
    println!("{}", addr);
    println!(
        "Send exactly {} satoshi amount of coins to the above address",
        contract.locked_asset_params.value
    );
}
/// Paramerters for the Option Asset being traded
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OptAssetParams {
    /// The asset being traded
    asset: AssetId,
    /// The options token for this address
    /// Beneficiary token or options claim token
    opt_tkn: AssetId,
    /// Value of the asset being traded
    value: u64,
}

impl OptAssetParams {
    pub fn new(asset: AssetId, opt_tkn: AssetId, value: u64) -> Self {
        Self {
            asset,
            opt_tkn,
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OptionContract {
    /// Time in bitcoin is represented as u32
    expiry: u32,
    /// The writer asset params
    locked_asset_params: OptAssetParams,
    /// The buyer asset params
    claim_asset_params: OptAssetParams,
    /// The control key
    /// This key does the check sig from stack operation and
    /// verifies the covenant
    /// In our construction this is public key whose private
    /// key is known by everyone. We can also use this to
    /// restrict the people we want to participate in the
    /// options contract
    control_key: PublicKey,
    /// The secret key corresponding to the above Pk
    /// This is not really a secret, but known to all participants
    control_sk: PrivateKey,
    /// The btc hard coded address
    /// Required for fees. Usually this would be the locked asset, but it's
    /// not necessary
    btc_asset: AssetId,
}

#[derive(Debug)]
pub enum Error {
    ExpectedExplicitAsset,
    MiniscriptErr(miniscript::Error),
    InvalidClaimTx,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ExpectedExplicitAsset => {
                write!(f, "Expected an Explicit Asset for creating Options")
            }
            Error::MiniscriptErr(ref e) => {
                write!(f, "Inner Miniscript Error: {}", e)
            }
            Error::InvalidClaimTx => {
                write!(
                    f,
                    "Fund transaction does not contain the locked collateral output"
                )
            }
        }
    }
}

#[doc(hidden)]
impl From<miniscript::Error> for Error {
    fn from(e: miniscript::Error) -> Error {
        Error::MiniscriptErr(e)
    }
}

impl error::Error for Error {}

// Create a txout spending to target spk with `value` amount of asset `asset`
fn txout(target_spk: Script, value: u64, asset: AssetId) -> TxOut {
    let mut tx_out = elements::TxOut::default();
    tx_out.script_pubkey = target_spk;
    tx_out.value = confidential::Value::Explicit(value);
    tx_out.asset = confidential::Asset::Explicit(asset);
    tx_out
}

// Check if the transaction outputs can be used for covenant operations
fn check_cov_txouts(tx: &Transaction) {
    let conf_txout_count = tx
        .output
        .iter()
        .filter(|x| x.nonce != confidential::Nonce::Null)
        .count();

    // Get the serialized size after blinding. Assume that txouts non-null nonce get blinded
    let ser_txout_len = serialize(&tx.output).len() + conf_txout_count * (33 - 9);
    // + 1 for var int len encoding that is not used in sighash calculation
    if ser_txout_len > 520 + 1 {
        panic!(
            "tx output len {} exceeds 520, try sending to a non-confidential address instead",
            ser_txout_len
        );
    }
}

impl OptionContract {
    pub fn from_config<'a>(matches: &clap::ArgMatches<'a>) -> Self {
        let expiry_date = matches
            .value_of("expiry")
            .expect("Expiry not provided")
            .parse::<NaiveDate>()
            .expect("Date format must YYYY-MM-DD");
        let expiry_date = Date::<Utc>::from_utc(expiry_date, Utc);

        let expiry_timestamp = expiry_date
            .and_time(NaiveTime::from_hms(0, 0, 0))
            .unwrap()
            .timestamp() as u32;

        let strike = matches
            .value_of("strike")
            .expect("Strike price(in USD) not provided")
            .parse::<f64>()
            .expect("Strike price format incorrect");
        let strike_amt = Amount::from_btc(strike)
            .expect("Strike amount must be positive")
            .as_sat();

        let out_path = matches.value_of("cfg-file").expect("Incorrect path string");
        let file = File::open(&out_path).expect(&format!("Config file not found at {}", out_path));
        let cfg: Config = serde_yaml::from_reader(file).expect("Malformed Config file");

        let locked_asset_params =
            OptAssetParams::new(cfg.lock_asset, cfg.bene_token, cfg.locked_asset_amount);
        let claim_asset_params = OptAssetParams::new(cfg.claim_asset, cfg.opt_token, strike_amt);

        let contract = OptionContract::new(
            expiry_timestamp,
            locked_asset_params,
            claim_asset_params,
            cfg.control_pk,
            cfg.control_sk,
            cfg.btc_asset,
        );
        contract
    }
    /// Create a new option
    /// In most cases. Claim Asset would be asset corresponding to usd
    pub fn new(
        expiry: u32,
        locked_asset_params: OptAssetParams,
        claim_asset_params: OptAssetParams,
        control_pk: bitcoin::PublicKey,
        control_sk: bitcoin::PrivateKey,
        btc_asset: AssetId,
    ) -> Self {
        Self {
            expiry,
            locked_asset_params,
            claim_asset_params,
            control_key: control_pk,
            control_sk: control_sk,
            btc_asset,
        }
    }

    // TxOut which burns the option token
    fn burn_opt(&self) -> TxOut {
        let mut tx_out = TxOut::default();
        tx_out.asset = confidential::Asset::Explicit(self.claim_asset_params.opt_tkn);
        tx_out.value = confidential::Value::Explicit(1);
        // Should we put some message?
        tx_out.script_pubkey = Script::new_op_return(&[]);
        tx_out
    }

    /// Returns a txout that burns the bene token.
    fn burn_bene(&self) -> TxOut {
        let mut tx_out = TxOut::default();
        tx_out.asset = confidential::Asset::Explicit(self.locked_asset_params.opt_tkn);
        tx_out.value = confidential::Value::Explicit(1);
        // Should we put some message?
        tx_out.script_pubkey = Script::new_op_return(&[]);
        tx_out
    }

    /// Create an wsh address which fixes the first output to the following
    /// Explicit Amount = 1; Asset = <explicit_asset>; Nonce= Null; ScriptPubkey = OP_RETURN
    /// Returns the wsh script pubkey and the corresponding txout
    fn burn_bene_desc(&self) -> (CovenantDescriptor<bitcoin::PublicKey>, TxOut) {
        let tx_out = self.burn_bene();
        let inner_ms = format!("outputs_pref({})", &serialize_hex(&tx_out));
        let ms = Miniscript::<PublicKey, Segwitv0>::from_str_insane(&inner_ms)
            .expect("Single txout less than 520 bytes");
        let desc = CovenantDescriptor::new(self.control_key, ms).unwrap();
        (desc, tx_out)
    }

    /// Returns a txout that burns the bene token.
    /// p2wsh wrapped op-return. Elements does not allow
    /// multi-op return per transaction
    fn burn_bene_wsh(&self) -> TxOut {
        let mut tx_out = TxOut::default();
        tx_out.asset = confidential::Asset::Explicit(self.locked_asset_params.opt_tkn);
        tx_out.value = confidential::Value::Explicit(1);
        // Should we put some message?
        tx_out.script_pubkey = Script::new_op_return(&[]).to_v0_p2wsh();
        tx_out
    }

    // Returns a pair of txouts. The first txout has a covenant constraint that it
    // can only be spend by transaction whose output at index 0 is the second txout.
    fn exercise_txout(&self) -> (TxOut, TxOut) {
        let (desc, txout2) = self.burn_bene_desc();
        let exercise_txout = txout(
            desc.script_pubkey(),
            self.claim_asset_params.value,
            self.claim_asset_params.asset,
        );
        (exercise_txout, txout2)
    }

    /// Helper function to create a descriptor
    pub fn deposit_desc(&self) -> Result<Descriptor<PublicKey>, Error> {
        // TxOut which burns the option
        let burn_opt_txout = self.burn_opt();
        let burn_bene_txout = self.burn_bene();
        let burn_bene_wsh_txout = self.burn_bene_wsh(); // a wsh wrapped op-return
        let (exercise_txout, _) = self.exercise_txout();

        // Create the three conditions of spending the output
        // As B fragments of Miniscript

        // 1. Create the expiry condition String
        // Burn the bene token to claim the expired option
        let expiry_cond = format!(
            "l:and_b(n:after({}),atv:outputs_pref({}))",
            self.expiry,
            serialize_hex(&burn_bene_txout),
        );

        // 2. Cancel the trade, burn both the token and free the coins
        // First output is burning opt, second one burns bene
        let cancel_cond = format!(
            "altv:outputs_pref({}{})",
            &serialize_hex(&burn_opt_txout),
            &serialize_hex(&burn_bene_wsh_txout),
        );

        // 3. Exercise Condition
        // The first output should burn the opt token and the second out
        // should send the claim asset amount to the benefit token holder
        let exercise_cond = format!(
            "altv:outputs_pref({}{})",
            &serialize_hex(&burn_opt_txout),
            &serialize_hex(&exercise_txout)
        );

        // Combine all three conditions using a thresh

        let inner_ms = format!(
            "thresh(1,{},{},{})",
            expiry_cond, cancel_cond, exercise_cond
        );
        let desc = Descriptor::<PublicKey>::from_str(&format!(
            "elcovwsh({},{})",
            self.control_key, inner_ms
        ))?;
        Ok(desc)
    }

    /// Get an address to deposit coins for this option
    /// Can error if the underlying covenant creation fails
    /// because of resource limits
    pub fn deposit_addr(
        &self,
        addr_params: &'static AddressParams,
    ) -> Result<elements::Address, Error> {
        let desc = self.deposit_desc()?;
        // println!("{}", serialize_hex(&desc.explicit_script()));
        let addr = desc.address(addr_params)?;
        Ok(addr)
    }

    /// Remove the locked asset from the contract
    pub fn claim_expiry(&self) -> Transaction {
        let out = self.burn_bene();
        Transaction {
            version: 2,
            lock_time: self.expiry + 1, // set the expiry. TODO: check if +1 is necessary
            input: vec![],
            output: vec![out],
        }
    }

    /// Second transaction for claiming expiry
    pub fn claim_expiry_tx2(
        &self,
        tx: &mut Transaction,
        contract_prevout: OutPoint,
        addr: Address,
    ) {
        let inp = TxIn {
            previous_output: contract_prevout,
            is_pegin: false,
            has_issuance: false,
            script_sig: Script::default(),
            sequence: 0,
            asset_issuance: AssetIssuance::default(),
            witness: TxInWitness::default(),
        };
        let nonce = addr
            .blinding_pubkey
            .map(confidential::Nonce::from)
            .unwrap_or(confidential::Nonce::Null);
        let mut tx_out = txout(
            addr.script_pubkey(),
            self.locked_asset_params.value,
            self.locked_asset_params.asset,
        );
        // Set the nonce to blinding key. Can be used with elements fundrawtransaction
        tx_out.nonce = nonce;

        tx.input.push(inp);
        tx.output.push(tx_out);

        // Need to re-arrange all outputs for covenant creation
        let burn_bene_pos = tx
            .output
            .iter()
            .position(|x| self.burn_bene() == *x)
            .expect("Tx must contain burn txout");
        tx.output.swap(burn_bene_pos, 0);

        check_cov_txouts(&tx);
    }

    /// The first transaction to send when cancelling the option
    pub fn cancel_opt(&self) -> Transaction {
        // Fill in a transaction with 0 inputs and 2 outputs
        // Input: None
        // Outputs: 1) The burned opt token
        //          2) The burned bene token wrapped in wsh

        let out1 = self.burn_opt();
        let out2 = self.burn_bene_wsh();
        let tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![],
            output: vec![out1, out2],
        };
        tx
    }

    /// Takes in the funded transaction as inputs and adds contract prevout and an output
    /// to claim the locked asset
    pub fn cancel_tx2(&self, tx: &mut Transaction, contract_prevout: OutPoint, addr: Address) {
        let inp = TxIn {
            previous_output: contract_prevout,
            is_pegin: false,
            has_issuance: false,
            script_sig: Script::default(),
            sequence: 0,
            asset_issuance: AssetIssuance::default(),
            witness: TxInWitness::default(),
        };
        let nonce = addr
            .blinding_pubkey
            .map(confidential::Nonce::from)
            .unwrap_or(confidential::Nonce::Null);
        let mut tx_out = txout(
            addr.script_pubkey(),
            self.locked_asset_params.value,
            self.locked_asset_params.asset,
        );
        // Set the nonce to blinding key. Can be used with elements fundrawtransaction
        tx_out.nonce = nonce;

        tx.input.push(inp);
        tx.output.push(tx_out);

        // Need to re-arrange all outputs for covenant creation
        let burn_opt_pos = tx
            .output
            .iter()
            .position(|x| self.burn_opt() == *x)
            .expect("Tx must contain burn txout");
        tx.output.swap(0, burn_opt_pos);
        let burn_bene_pos = tx
            .output
            .iter()
            .position(|x| self.burn_bene_wsh() == *x)
            .expect("Tx must contain burn wsh wrapped op-return bene token txout");
        tx.output.swap(burn_bene_pos, 1);

        check_cov_txouts(&tx);
    }

    /// Incase the option is exercised, claim the corresponding usd amount
    /// Creates a raw transaction that burns the bene token
    pub fn claim_bene(&self) -> Transaction {
        let mut tx = self.claim_expiry();
        // claiming using bene token is the same as expiry except that we claim using
        // different asset and we don't set a locktime
        tx.lock_time = 0;
        tx
    }

    /// Incase the option is exercised, claim the corresponding usd amount
    /// Creates a raw transaction that burns the bene token
    pub fn claim_bene_tx2(&self, tx: &mut Transaction, prevout: OutPoint, addr: Address) {
        // Calling blindrawtransaction does not work directly because elements wallet
        // does not lookup the blockchain for explicit txouts for blinding.
        // Technically, for explicit txouts the wallet can lookup the blockchain and blind
        // the output. But currently blinding fails for outputs whose asset does
        // not any input spending input that is IsMine.
        if addr.blinding_pubkey.is_some() {
            panic!("Receiving address cannot be confidential")
        }

        self.claim_expiry_tx2(tx, prevout, addr);
        // claiming using bene token is the same as expiry except that we claim using
        // different asset

        let last_out = tx.output.last_mut().unwrap();
        last_out.asset = confidential::Asset::Explicit(self.claim_asset_params.asset);
        last_out.value = confidential::Value::Explicit(self.claim_asset_params.value);
    }

    /// Get the transaction to broadcast at exercise
    /// Forward this transaction to elementsd for fundrawtransaction
    pub fn exercise_opt_tx2(
        &self,
        tx: &mut Transaction,
        contract_prevout: OutPoint,
        addr: Address,
    ) {
        let inp = TxIn {
            previous_output: contract_prevout,
            is_pegin: false,
            has_issuance: false,
            script_sig: Script::default(),
            sequence: 0,
            asset_issuance: AssetIssuance::default(),
            witness: TxInWitness::default(),
        };
        let nonce = addr
            .blinding_pubkey
            .map(confidential::Nonce::from)
            .unwrap_or(confidential::Nonce::Null);
        let mut tx_out = txout(
            addr.script_pubkey(),
            self.locked_asset_params.value,
            self.locked_asset_params.asset,
        );
        // Set the nonce to blinding key. Can be used with elements fundrawtransaction
        tx_out.nonce = nonce;

        tx.input.push(inp);
        tx.output.push(tx_out);

        // Need to re-arrange all outputs for covenant creation
        let burn_pos = tx
            .output
            .iter()
            .position(|x| self.burn_opt() == *x)
            .expect("Tx must contain burn txout");
        tx.output.swap(0, burn_pos);
        let exercise_pos = tx
            .output
            .iter()
            .position(|x| self.exercise_txout().0 == *x)
            .expect("Tx must contain burn txout");
        tx.output.swap(exercise_pos, 1);

        check_cov_txouts(&tx);
    }

    /// Get the transaction to broadcast at exercise
    /// Forward this transaction to elementsd for fundrawtransaction
    pub fn finalize_tx(&self, tx: &mut Transaction, ty: &str) {
        // Miniscript Magic
        use elements::pset::PartiallySignedTransaction as Pset;
        println!("{}", tx.txid());
        let mut pset = Pset::from_tx(tx.clone());
        println!("{}", pset.extract_tx().unwrap().txid());
        // dbg!(&tx);
        let tx2 = pset.extract_tx().unwrap();
        assert_eq!(tx.version, tx2.version);
        assert_eq!(tx.input, tx2.input);
        // assert_eq!(tx.output[0], tx2.output[0]);
        // assert_eq!(tx.output[1], tx2.output[1]);
        assert_eq!(tx.output[2].asset, tx2.output[2].asset);
        assert_eq!(tx.output[2].value, tx2.output[2].value);
        assert_eq!(tx.output[2].nonce, tx2.output[2].nonce);
        assert_eq!(tx.lock_time, tx2.lock_time);

        let secp = elements::secp256k1_zkp::Secp256k1::new();
        let cov_index = pset.inputs.len() - 1;
        let cov_in = &mut pset.inputs[cov_index];

        // Get the sighash script code and value based on the tx we are spending
        // In expiry, cancel and exercise cases we are spending from covenant prevout
        // But in claimbene case, we are spending the exercise transaction
        let (script_code, value) = if ty == "expiry" || ty == "cancel" || ty == "exercise" {
            // The descriptor must be a covenant descriptor
            let desc = self.deposit_desc().unwrap();
            let desc = desc.as_cov().expect("Must be a cov descriptor");
            cov_in.witness_script = Some(desc.explicit_script());
            cov_in.witness_utxo = Some(txout(
                desc.script_pubkey(),
                self.locked_asset_params.value,
                self.locked_asset_params.asset,
            ));
            (
                desc.cov_script_code(),
                confidential::Value::Explicit(self.locked_asset_params.value),
            )
        } else if ty == "claimbene" {
            let (desc, _) = self.burn_bene_desc();
            cov_in.witness_script = Some(desc.explicit_script());
            cov_in.witness_utxo = Some(txout(
                desc.script_pubkey(),
                self.claim_asset_params.value,
                self.claim_asset_params.asset,
            ));
            (
                desc.cov_script_code(),
                confidential::Value::Explicit(self.claim_asset_params.value),
            )
        } else {
            unreachable!("type paramter must be valid cancel/expiry/claimbene/exercise");
        };

        // Create a signature for covenant operation
        let mut sighash_cache = elements::sighash::SigHashCache::new(&*tx);
        println!("tx: {}", tx.txid());
        let sighash_type = cov_in.sighash_type.unwrap_or(SigHashType::All);
        let sighash_msg =
            sighash_cache.segwitv0_sighash(cov_index, &script_code, value, sighash_type);

        let msg = secp256k1_zkp::Message::from_slice(&sighash_msg[..]).expect("32 byte sighash");
        let sig = secp.sign(&msg, &self.control_sk.key);

        let rawsig = elements_miniscript::elementssig_to_rawsig(&(sig, sighash_type));
        cov_in.partial_sigs.insert(self.control_key, rawsig);

        miniscript::pset::finalize_input(&mut pset, &secp, cov_index).expect("Miniscript error");

        *tx = pset.extract_tx().unwrap();
    }

    /// Get the transaction to broadcast at exercise
    /// Forward this transaction to elementsd for fundrawtransaction
    pub fn exercise_opt(&self) -> Transaction {
        // Fill in a transaction with 0 inputs and 2 outputs
        // Input: None
        // Outputs: 1) The burned opt token
        //          2) The btc amount sent to requested address

        let out1 = self.burn_opt();
        let (out2, _) = self.exercise_txout();
        let tx = Transaction {
            version: 2,
            lock_time: 0,
            input: vec![],
            output: vec![out1, out2],
        };
        tx
    }
}

#[cfg(test)]
mod tests {
    use elements::secp256k1_zkp;
    use std::str::FromStr;

    use super::*;
    #[test]
    fn test_keys() {
        let priv_key = bitcoin::PrivateKey::from_wif(CTRL_SK).expect("Known private key");
        let secp = secp256k1_zkp::Secp256k1::signing_only();
        let pk = bitcoin::PublicKey::from_str(CTRL_PK).unwrap();
        assert_eq!(pk, priv_key.public_key(&secp));
    }
}
