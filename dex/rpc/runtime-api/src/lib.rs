#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_std::vec::Vec;

use zenlink_dex::{ExchangeInfo, TokenInfo};

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait ZenlinkDexApi<AccountId, AssetId, TokenBalance, Balance, ExchangeId>
    where
        AccountId: Codec,
        AssetId: Codec,
        TokenBalance: Codec,
        Balance: Codec,
        ExchangeId: Codec,
    {
        fn get_token_info(token_id: AssetId) -> Option<TokenInfo<TokenBalance>>;
        fn get_token_balance(token_id: AssetId, owner: AccountId) -> TokenBalance;
        fn get_token_allowance(token_id: AssetId, owner: AccountId, spender: AccountId) -> TokenBalance;
        fn get_exchange_by_token_id(token_id: AssetId) -> Option<ExchangeInfo<AccountId, AssetId, TokenBalance, Balance, ExchangeId>>;
        fn get_exchange_by_id(id: ExchangeId) -> Option<ExchangeInfo<AccountId, AssetId, TokenBalance, Balance, ExchangeId>>;
        // TODO：Pagination
        fn get_exchanges() -> Vec<ExchangeInfo<AccountId, AssetId, TokenBalance, Balance, ExchangeId>>;
    }
}
