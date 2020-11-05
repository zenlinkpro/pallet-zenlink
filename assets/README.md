# zenlink-assets

## overview
The implement of the ERC20 assets.

## variants

##### 1. issue(`T::TokenBalance`, `AssetInfo`)

```text
Issue a new class of pallet-zenlink assets. There are, and will only ever be, 
total such assets and they'll all belong to the origin initially. 
It will have an identifier AssetId instance: 
this will be specified in the Issued event.
```
    
- `total`: initial total supply.
- `asset_info`: the asset info contains name, symbol, decimals.

##### 2. transfer`(T::AssetId, <T::Lookup as StaticLookup>::Source, T::TokenBalance)`

```text
Move some assets from one holder to another.
```
- `id`: the asset id.
- `target`: the receiver of the asset.
- `amount`: the amount of the asset to transfer.

##### 3. approve`(T::AssetId, <T::Lookup as StaticLookup>::Source, T::TokenBalance)`

```text
Allow spender to withdraw from the origin account.
```

- `id`: the asset id.
- `spender`: the spender account.
- `amount`: the amount of allowance.

##### 4. transfer_from`(T::AssetId, <T::Lookup as StaticLookup>::Source, <T::Lookup as StaticLookup>::Source, T::TokenBalance)`

```text
Send amount of asset from Account from to Account target.
```

- `id`: the asset id.
- `from`: the source of the asset to be transferred.
- `target`: the receiver of the asset to be transferred.
- `amount`: the amount of asset to be transferred.

## beyond ERC20 for zenlink-dex 

- mint: `Increase the total supply of the asset`

```
pub fn inner_mint(
    id: &T::AssetId,
    owner: &T::AccountId,
    amount: T::TokenBalance
) -> DispatchResult
```

- burn: `Decrease the total supply of the asset`

```
pub fn inner_burn(
    id: &T::AssetId,
    owner: &T::AccountId,
    amount: T::TokenBalance
) -> DispatchResult
```


