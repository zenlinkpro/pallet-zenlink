# pallet-zenlink

## overview
pallet-zenlink is a set of the pallets that implement 
the [Zenlink Protocol](https://zenlink.pro/) which is `A cross-chain DEX network based on Polkadot`

- [zenlink-assets](./assets/README.md): the implement of the ERC20 assets.
- [zenlink-dex](./dex/README.md): the implement of the Uniswap v1 functionality. it is closely coupled to the `zenlink-assets`

## work flow
- issue some tokens which are ERC20 assets by `zenlink-assets` module.
- create the exchange between native currency and tokens by `zenlink-dex` module.
- add some liquidity to the exchange by `zenlink-dex` module.
- swap in the `currency-token`,`token-currency` and `token-token` exchanges by `zenlink-dex` module.

## tests

```bash
cargo test
```

## license
under Apache License v2
