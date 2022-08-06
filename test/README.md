# Differential privacy test

## Starting the dev chain

The contract is written with the beta cosmwasm 1.0 updates for secret network, so you need to run the following version of the dev chain:

```sh
docker run -it --rm -p 9091:9091 -p 26657:26657 -p 1317:1317 -p 5000:5000 --name localsecret ghcr.io/scrtlabs/localsecret:v1.4.0-cw-v1-beta.2
```

- Port 9091 - gRPC-web - For secretjs@beta
- Port 26657 - RPC - For secretcli
- Port 1317 - LCD - For old secretjs
- Port 5000 - Faucet - E.g. curl "http://localhost:5000/faucet?address=${SECRET_ADDRESS}"

## Install packages

```sh
npm install
```

## Run the tests

```sh
npm run start
```