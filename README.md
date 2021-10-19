# Burrowland contact

## Development

### Build (optional)

Requires Rust and wasm32 target.

```bash
./build.sh
```

### Deploy on the testnet

Requires NEAR CLI to be installed.

```bash
./scripts/dev_deploy.sh
```

This will provide a list of exports at the end. Execute them to get the CLI ready for further transactions.

Example exports:
```bash
export NEAR_ENV=testnet
export OWNER_ID=dev-1634411537975-18277461139961
export ORACLE_ID=dev-1634411553736-97024242878560
export CONTRACT_ID=dev-1634411561699-94876475207945
export BOOSTER_TOKEN_ID=ref.fakes.testnet
export WETH_TOKEN_ID=weth.fakes.testnet
export DAI_TOKEN_ID=dai.fakes.testnet
export USDT_TOKEN_ID=usdt.fakes.testnet
export WNEAR_TOKEN_ID=wrap.testnet
export ONE_YOCTO=0.000000000000000000000001
export GAS=200000000000000
```

### Create a test account

Requires exports from running `dev_deploy`. 

```bash
./scripts/create_account.sh
```

This will create a new test account with fake assets. Execute the export at the end to get the account ID.





