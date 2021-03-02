# options-elements
Write call/put options on elements using elements-miniscript. This tool does
*NOT* have access to the wallet/blockchain. It will help you create/sign transactions
with help of elements-cli.

# General Info

Options are financial derivatives that give
buyers the right, but not the obligation, to buy or sell an underlying asset at
an agreed-upon price and date. These are commonly known as strike price and expiry date.
Similarly, in elements we can create options using covenants and assets. Options can be
created between any two assets, however for simplicity we assume bitcoin-lusd call options.

For issusing options in elements, we need additional token assets(assets with 1 unit/satoshi)
to represent the ownership of option. We refer  to `opt-token` as token that represents the
ownership of option. And `bene-token` which represents the other side(ownership of the locked btc).
These tokens can be traded freely to transfer the long/short side of the option.
For security of the system, it is important these `opt-token` and `bene-token` have exactly
1 unit of supply with no reissuance. It is the responsibility of the user to make sure
that these are issued correctly.

*NOTE:* The `opt-token` does not need to 1 unit of supply. It's supply needs to match the number of option-tokens.
Throughout the codebase, the term locked asset is represented asset that is locked in options contract and the
term claim asset is used to for the asset that is used in order to exercise the option. For ex, in bitcoin call
option for 1BTC 50k$, the locked asset would be native btc asset and claim asset would be l-usd asset.

# Using option-elements
We first need to initialize the `options-elements` with respective parameters for
l-usd(`claim-asset`) asset, `bene-token`, `lock-asset`, `opt-token`. These options
are saved by default in `opt_cfg.conf` file in the current directory. You can either
edit create the file(YAML) manually or use the `init` command for creating it.
```
sanket1729@sanket-pc:~/options-elements$ more opt_cfg.conf
---
lock_asset: b2e15d0d7a0c94e4e2ce0fe6e8691b9e451377f6e46e8045a86f7c4b5d4f0f23
btc_asset: b2e15d0d7a0c94e4e2ce0fe6e8691b9e451377f6e46e8045a86f7c4b5d4f0f23
claim_asset: e855b6b5cfa07860a6df2f6174c51a4c1408a75d228e674d88d7522c3ca15362
opt_token: d37963f81b572b2726a0fed03507b693f7ee2b6b0270cf02b30e73b71a2f231d
bene_token: 75b8fb4cf7fb147f5d31cc47c3e45656e6424b60b5f407cbe41afa298898277b
locked_asset_amount: 100000000
control_pk: 039b6347398505f5ec93826dc61c19f47c66c0283ee9be980e29ce325a0f4679ef
control_sk: cVt4o7BGAig1UXywgGSmARhxMdzP5qvQsxKkSsc1XEkw3tDTQFpy
```
All paramters in the file can be edited. The `locked_asset_amount` represents the amount of
asset locked in when creating a call option. The `init` command takes in some default values
for the configuration that can supplied as optional arguments. See `init --help` for details.
```
options-elements init [FLAGS] [OPTIONS] --bene-token <bene-token> --claim-asset <claim-asset> --lock-asset <lock-asset> --opt-token <opt-token>
```
**NOTE:** All interactions with options-elements binary would read this conf file and would for this
file in the current directory. All commands have a option to provide a location for reading this file
incase you don't save it at a default location.

# Creating Call options:

After initializing, we have fixed what `bene-token`, `opt-token`, `locked-asset`, `claim-asset` we are dealing with.
This process only one options-contract at a time, therefore it is important to maintain separate files per options contract.
Call options can be created by `options-elements call create`. See `options-elements call create --help` for details.

```
options-elements call create [FLAGS] [OPTIONS] --expiry <expiry> --strike <strike>
```
Since we have fixed all assets and tokens, options are now similar to regular stock options, but that can be
identified by expiry and strike.

# Operations on Options Contracts:

There are three major operations supported on options contract.
1. Exercise
2. Cancel
3. Expiry

We also a fourth operation called `claimbene` that is used for claiming the lusd incase the option is exercised.
4.

The general operation of the app is as follows.(assuming the app has a proper cfg file
and it is option contract has been created). See `--help` for more information on args.
1. Create a raw transaction using `options-elements exercise <args>`. This creates a partial transaction
that can be passed to `fundrawtransaction`. We need to do this in two steps because the elements `fundrawtransction`
does not understand covenant descriptors and hence it cannot infer the fees for satisfaction. So, initially we pass
a transaction without the covenant prevout and the corresponding output. This steps outputs a transaction with a dummy fee
rate that can be used in regtest mode.
Ex: Assuming default location of cfg file, expiry = 2021-05-30, strike = 100.0
```
./target/debug/options-elements call exercise --expiry=2021-05-30 --strike=100.0
```
2. Pass the funded the to elements-cli
```
FUNDED=$(e1-cli fundrawtransaction $RAW '''{"feeRate": 0.03}''' | jq '.hex' | tr -d '"')
```
The jq utility parses the output json and tr trims the `"` symbol.

3. This step is same for all operations of `exercise`, `expiry`, `cancel` and `claimbene`.
This adds information about contract prevout, the type of operation and receiving address.
Since we add the same amount to input and output the transaction amounts are still balanced.
This outputs a unblinded transcation, a list of auxillary generators that can be passed
to `blindrawtransaction`.
Note when using this for claiming l-usd when the option is exercised, the prevout information
for the exercise transaction must be supplied instead of the contract prevout.
```
./target/debug/options-elements call addcontract --expiry=2021-05-30 --strike=100.0 --addr=$ADDR --prev-txid=$TXID --prev-vout=$VOUT --type=exercise --funded-tx=$FUNDED
```

4. Blind the tx. Note that Step 3 also outputs aux gen list that is to supplied as
third parameter. This blinds all outputs of the transaction that are marked for blinding

```
BLINDED=$(e1-cli blindrawtransaction $UNBLINDED true  <aux_gen_list>)
```

5. Sign the transaction. This will only sign inputs that are owned by the wallet. This cannot finalize the covenant input
which we will do in final step.
```
SIGNED=$(e1-cli signrawtransactionwithwallet $BLINDED | jq '.hex' | tr -d '"')
```

6. Finalize the transaction. This does all the covenant miniscript magic and satisfies the covenant input.
Outputs a raw hex can be sent by `sendrawtransaction`
```
./target/debug/options-elements call finalize --expiry=2021-05-18 --strike=100.0 --signed-tx=$SIGNED --type=exercise
```

7. Send the tx
```
e1-cli sendrawtransaction <hex>
```
# Cancel, Expiry and Claiming Bene

Claiming expiry, cancelling and claiming bene only differ in the first step(Step 1 above).
1. expiry `options-elements call expiry <args>`
2. cancel `options-elements call cancel <args>`
3. claimbene `options-elements call claimbene <args>`

While Steps 2 -6 remain the same, user must be pass the type of operation `--type=exercise/cancel/claimbene/expiry`
accordingly.