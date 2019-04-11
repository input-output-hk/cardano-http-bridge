# Cardano HTTP Bridge

[![Build Status](https://travis-ci.org/input-output-hk/cardano-http-bridge.svg?branch=master)](https://travis-ci.org/input-output-hk/cardano-http-bridge)

The cardano HTTP bridge provides a JSON REST API to query:

* Blocks;
* Pack of blocks (Organized by epochs);
* Protocol genesis configuration file;
* TIP (the latest work of the network);
* UTxOs

And to post transactions

# How to build

1. you need to [install the rust toolchain](https://www.rust-lang.org/tools/install);
2. you need to build the project: `cargo run --release`

# How to start a new http bridge instance

You are interested only about the `start` command:

Options:

* `--networks-dir <NETWORKS DIRECTORY>`    the relative or absolute directory of the networks to server, default is under the `${HOME}/.hermes/networks/` directory
* `--port <PORT NUMBER>`                   set the port number to listen to [default: 80]
* `--template <TEMPLATE>...`               either 'mainnet' or 'testnet'; may be given multiple times [default: mainnet]  [possible values: mainnet, staging, testnet]

Example, if you wish the http-bridge to server mainnet and staging:

```
cardano-http-bridge start --port=80 --template=mainnet,staging
```

# Offered APIs:

## GET: `/:network/block/:blockid` query block

This allows to query a block in its binary format.

* `:network` is any of the network passed to the `--template` options at startup.
* `:blockid` the hash identifying a block within the blockchain

Example:

```
wget http://localhost:8080/mainnet/block/6abb9309dd72dd5901fc6dad22caaefc15bd08d5f297503001a9efdaee1eec2b
```

## GET: `/:network/epoch/:epochid`

This allows you to query a whole epoch in its binary format.

* `:network` is any of the network passed to the `--template` options at startup.
* `:epochid` the epoch number (0, 1, 2 ...)

Example:

```
wget http://localhost:8080/mainnet/epoch/2
```

## GET: `/:network/genesis/:hash`

This allows you to query a genesis file, if you know the hash of the genesis file you can query it here:

* `:network` is any of the network passed to the `--template` options at startup.
* `:hash` the hash of the genesis file

## GET: `/:network/tip`

Download the block header (binary format) of the TIP of the blockchain: the latest known block.

* `:network` is any of the network passed to the `--template` options at startup.

Example:

```
wget http://localhost:8080/mainnet/tip
```

## POST: `/:network/txs/signed`

Allows you to send a signed transaction to the network. The transaction will then be
disseminated to the different nodes it knows of:

* `:network` is any of the network passed to the `--template` options at startup.

The body of the request is the base64 encoded signed transaction.

## GET: `/:network/utxos/:address`

Allows you to query utxos in JSON format given:

* `:network` is any of the network passed to the `--template` options at startup.
* `:address` base58 encoding of an address

Example query:

```
curl http://localhost:8080/mainnet/utxos/2cWKMJemoBamE3kYCuVLq6pwWwNBJVZmv471Zcb2ok8cH9NjJC4JUkq5rV5ss9ALXWCKN
```

Possible response:
```json
[
    {
        "address": "2cWKMJemoBamE3kYCuVLq6pwWwNBJVZmv471Zcb2ok8cH9NjJC4JUkq5rV5ss9ALXWCKN",
        "coin": 310025,
        "index": 0,
        "txid": "89eb0d6a8a691dae2cd15ed0369931ce0a949ecafa5c3f93f8121833646e15c3"
    }
]
```

## GET: `/:network/chain-state/:epochid`

## GET: `/:network/chain-state-delta/:epochid/:to`
