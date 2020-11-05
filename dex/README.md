# zenlink-dex

## overview
 Built-in decentralized exchange modules in Substrate 2.0 network, 
 the swap mechanism refers to the design of Uniswap V1.
 
## variants

##### 1. create_exchange`(AssetId)`
```
Create an exchange with the token which would swap with native currency
```

- `token_id`: The exist asset's id.

##### 2. add_liquidity`(SwapHandler, Balance, TokenBalance, TokenBalance, BlockNumber)`

```
Injecting liquidity to specific exchange liquidity pool 
in the form of depositing currencies to the exchange account and 
issue liquidity pool token in proportion to the caller who is the liquidity provider. 
The liquidity pool token, shares ZLK, allowed to transfer 
but can't swap in exchange it represents the proportion of assets in liquidity pool.
```

- `swap_handler`: The wrapper of exchangeId and assetId to access.
- `currency_amount`: Amount of base currency to lock.
- `min_liquidity`: Min amount of exchange shares(ZLK) to create.
- `max_token`: Max amount of token to input.
- `deadline`: When to invalidate the transaction.

##### 3. remove_liquidity`(SwapHandler, TokenBalance, Balance, TokenBalance, BlockNumber)`

```
Remove liquidity from specific exchange liquidity pool in the form of burning shares(ZLK), 
and withdrawing currencies from the exchange account in proportion, 
and withdraw liquidity incentive interest.
```

- `swap_handler`: The wrapper of exchangeId and assetId to access.
- `zlk_to_burn`: Liquidity amount to remove.
- `min_currency`: Minimum currency to withdraw.
- `min_token`: Minimum token to withdraw.
- `deadline`: When to invalidate the transaction.

##### 4. currency_to_token_input`(SwapHandler, Balance, TokenBalance, BlockNumber, AccountId)`
```
Swap currency to token.

User specifies the exact amount of currency to sold and 
the amount not less the minimum token to be returned.
```

- `swap_handler`: The wrapper of exchangeId and assetId to access.
- `currency_sold`: The balance amount to be sold.
- `min_token`: The minimum token expected to buy.
- `deadline`: When to invalidate the transaction.
- `recipient`: Receiver of the bought token.

##### 5. currency_to_token_output`(SwapHandler, TokenBalance, Balance, BlockNumber, AccountId)
```
Swap currency to token.

User specifies the maximum currency to be sold and 
the exact amount of token to be returned.
```

- `swap_handler`: The wrapper of exchangeId and assetId to access.
- `tokens_bought`: The amount of the token to buy.
- `max_currency`: The maximum currency expected to be sold.
- `deadline`: When to invalidate the transaction.
- `recipient`: Receiver of the bought token.

##### 6. token_to_currency_input`(SwapHandler, TokenBalance, Balance, BlockNumber, AccountId)
```
Swap token to currency.

User specifies the exact amount of token to sold and 
the amount not less the minimum currency to be returned.
```

- `swap_handler`: The wrapper of exchangeId and assetId to access.
- `token_sold`: The token balance amount to be sold.
- `min_currency`: The minimum currency expected to buy.
- `deadline`: When to invalidate the transaction.
- `recipient`: Receiver of the bought currency.

##### 7. token_to_currency_output(SwapHandler, Balance, TokenBalance, BlockNumber, AccountId)
```
Swap token to currency.

User specifies the maximum token to be sold and 
the exact currency to be returned.
```

- `swap_handler`: The wrapper of exchangeId and assetId to access.
- `currency_bought`: The balance of currency to buy.
- `max_token`: The maximum currency expected to be sold.
- `deadline`: When to invalidate the transaction.
- `recipient`: Receiver of the bought currency.

##### 8. token_to_token_input(SwapHandler, SwapHandler, TokenBalance, TokenBalance, BlockNumber, AccountId)
```
Swap token to other token.

User specifies the exact amount of token to sold and 
the amount not less the minimum other token to be returned.
```

- `swap_handler`: The wrapper of exchangeId and assetId to access.
- `other_swap_handle`: The wrapper of exchangeId and assetId to access..
- `token_sold`: The token balance amount to be sold.
- `min_other_token`: The minimum other token expected to buy.
- `deadline`: When to invalidate the transaction.
- `recipient`: Receiver of the bought other token.

##### 9. token_to_token_output(SwapHandler, SwapHandler, TokenBalance, TokenBalance, BlockNumber, AccountId)

```
Swap token to other token.

User specifies the maximum token to be sold and 
the exact other token to be returned.
```

- `swap_handler`: The wrapper of exchangeId and assetId to access.
- `other_swap_handle`: The wrapper of exchangeId and assetId to access..
- `other_token_bought`: The amount of the other token to buy.
- `max_token`: The maximum token expected to be sold.
- `deadline`: When to invalidate the transaction.
- `recipient`: Receiver of the bought currency.