# Postgress Geyser Plugin

**This is not an offical or endorsed list and is not affiliated with Solana Labs or the Solana Foundation in any way.**

The Solana Geyser interface is a handy way to access both account writes, blocks and (in 1.9) transactions as they are processed by the validator. 
This plugin is a simple one which sends transactions to a Postgres database using diesel

## how to run
**Make sure you have solana-testnet-cli installed and using mac**

run the database for postgres:

```
docker compose up
```
now run script

```
./run.sh
```
