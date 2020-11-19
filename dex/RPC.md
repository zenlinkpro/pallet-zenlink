# zenlink-dex-rpc

##### 1. zenlinkDex_getTokenInfo
get the token info by token Id.
- `at`: the specified block hash.
- `token_id`: the asset id of the token.

```rust
#[rpc(name = "zenlinkDex_getTokenInfo")]
    fn get_token_info(
        &self,
        at: Option<BlockHash>,
        token_id: AssetId,
    ) -> Result<Option<TokenInfo<
        TokenBalance
    >>>;
```

```bash
$ curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
     "jsonrpc":"2.0",
      "id":1,
      "method":"zenlinkDex_getTokenInfo",
      "params": [null, 0]
    }'
```

##### 2. zenlinkDex_getTokenBalance
get the balance of the token which the owner own
- `at`: the specified block hash.
- `token_id`: the asset id of the token.
- `owner`: the token's owner.

```rust
#[rpc(name = "zenlinkDex_getTokenBalance")]
    fn get_token_balance(
        &self,
        at: Option<BlockHash>,
        token_id: AssetId,
        owner: AccountId,
    ) -> Result<TokenBalance>;
```

```bash
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
     "jsonrpc":"2.0",
      "id":1,
      "method":"zenlinkDex_getTokenBalance",
      "params": [null, 0, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"]
    }'
```
##### 3. zenlinkDex_getTokenAllowance
get the allowance of the owner's asset
- `at`: the specified block hash.
- `token_id`: the asset id of the token.
- `owner`: the token's owner.
- `spender`: the allowance's spender.

```rust
#[rpc(name = "zenlinkDex_getTokenAllowance")]
    fn get_token_allowance(
        &self,
        at: Option<BlockHash>,
        token_id: AssetId,
        owner: AccountId,
        spender: AccountId,
    ) -> Result<TokenBalance>;
```

```bash
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
     "jsonrpc":"2.0",
      "id":1,
      "method":"zenlinkDex_getTokenAllowance",
      "params": ["0x2ad73d303c6ee0c2dda90e089b7782ca9db567a4a66e0a25524ff5ab1192f81e,", 0, "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY", "5EYCAe5kjMUvmw3KJBswvhJKJEJh4v7FdzqtsQnc9KtK3Fxk"]
    }'
```
##### 4. zenlinkDex_getExchangeByTokenId
get the exchange info by the token id.
- `at`: the specified block hash.
- `token_id`: the asset id of the token.

```rust
#[rpc(name = "zenlinkDex_getExchangeByTokenId")]
    fn get_exchange_by_token_id(
        &self,
        at: Option<BlockHash>,
        token_id: AssetId,
    ) -> Result<Option<ExchangeInfo<
        AccountId,
        AssetId,
        TokenBalance,
        RpcU128<Balance>,
        ExchangeId
    >>>;
```

```bash
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
     "jsonrpc":"2.0",
      "id":1,
      "method":"zenlinkDex_getExchangeByTokenId",
      "params": [null, 0]
    }'
```
##### 5. zenlinkDex_getExchangeById
get the exchange info by the exchange id.
- `at`: the specified block hash.
- `id`: the specified exchange id.

```rust
#[rpc(name = "zenlinkDex_getExchangeById")]
    fn get_exchange_by_id(
        &self,
        at: Option<BlockHash>,
        id: ExchangeId,
    ) -> Result<Option<ExchangeInfo<
        AccountId,
        AssetId,
        TokenBalance,
        RpcU128<Balance>,
        ExchangeId
    >>>;
```

```bash
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
     "jsonrpc":"2.0",
      "id":1,
      "method":"zenlinkDex_getExchangeById",
      "params": [null, 0]
    }'
```
##### 6. zenlinkDex_getExchanges
retrieve all exchanges info
- `at`: the specified block hash.

```rust
#[rpc(name = "zenlinkDex_getExchanges")]
    fn get_exchanges(
        &self,
        at: Option<BlockHash>,
    ) -> Result<Vec<ExchangeInfo<
        AccountId,
        AssetId,
        TokenBalance,
        RpcU128<Balance>,
        ExchangeId
    >>>;
```

```bash
curl http://localhost:9933 -H "Content-Type:application/json;charset=utf-8" -d   '{
     "jsonrpc":"2.0",
      "id":1,
      "method":"zenlinkDex_getExchanges",
      "params": []
    }'
```