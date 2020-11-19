//! RPC interface for the zenlink dex module.
#![allow(clippy::type_complexity)]

#[cfg(feature = "std")]
use std::{
    fmt::{Debug, Display},
    result::Result as StdResult,
    str::FromStr,
};
use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
#[cfg(feature = "std")]
use serde::{de, Deserialize, ser, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use zenlink_dex::{Exchange, ExchangeInfo, TokenInfo};
use zenlink_dex_runtime_api::ZenlinkDexApi as ZenlinkDexRuntimeApi;

/// A helper struct for handling u128 serialization/deserialization of RPC.
/// See https://github.com/polkadot-js/api/issues/2464 for details (shit!).
#[derive(Eq, PartialEq, Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RpcU128<T: Display + FromStr>(#[serde(with = "self::serde_num_str")] T);

impl<T: Display + FromStr> From<T> for RpcU128<T> {
    fn from(value: T) -> Self {
        RpcU128(value)
    }
}

/// Number string serialization/deserialization
pub mod serde_num_str {
    use super::*;

    /// A serializer that encodes the number as a string
    pub fn serialize<S, T>(value: &T, serializer: S) -> StdResult<S::Ok, S::Error>
        where
            S: ser::Serializer,
            T: Display,
    {
        serializer.serialize_str(&value.to_string())
    }

    /// A deserializer that decodes a string to the number.
    pub fn deserialize<'de, D, T>(deserializer: D) -> StdResult<T, D::Error>
        where
            D: de::Deserializer<'de>,
            T: FromStr,
    {
        let data = String::deserialize(deserializer)?;
        data.parse::<T>()
            .map_err(|_| de::Error::custom("Parse from string failed"))
    }
}

#[rpc]
pub trait ZenlinkDexApi<
    BlockHash,
    AccountId,
    AssetId,
    TokenBalance,
    Balance,
    ExchangeId
> where
    Balance: Display + FromStr,
    TokenBalance: Display + FromStr,
{
    #[rpc(name = "zenlinkDex_getTokenInfo")]
    fn get_token_info(
        &self,
        at: Option<BlockHash>,
        token_id: AssetId,
    ) -> Result<Option<TokenInfo<
        RpcU128<TokenBalance>
    >>>;

    #[rpc(name = "zenlinkDex_getTokenBalance")]
    fn get_token_balance(
        &self,
        at: Option<BlockHash>,
        token_id: AssetId,
        owner: AccountId,
    ) -> Result<RpcU128<TokenBalance>>;

    #[rpc(name = "zenlinkDex_getTokenAllowance")]
    fn get_token_allowance(
        &self,
        at: Option<BlockHash>,
        token_id: AssetId,
        owner: AccountId,
        spender: AccountId,
    ) -> Result<RpcU128<TokenBalance>>;

    #[rpc(name = "zenlinkDex_getExchangeByTokenId")]
    fn get_exchange_by_token_id(
        &self,
        at: Option<BlockHash>,
        token_id: AssetId,
    ) -> Result<Option<ExchangeInfo<
        AccountId,
        AssetId,
        RpcU128<TokenBalance>,
        RpcU128<Balance>,
        ExchangeId
    >>>;

    #[rpc(name = "zenlinkDex_getExchangeById")]
    fn get_exchange_by_id(
        &self,
        at: Option<BlockHash>,
        id: ExchangeId,
    ) -> Result<Option<ExchangeInfo<
        AccountId,
        AssetId,
        RpcU128<TokenBalance>,
        RpcU128<Balance>,
        ExchangeId
    >>>;

    // TODO：Pagination
    #[rpc(name = "zenlinkDex_getExchanges")]
    fn get_exchanges(
        &self,
        at: Option<BlockHash>,
    ) -> Result<Vec<ExchangeInfo<
        AccountId,
        AssetId,
        RpcU128<TokenBalance>,
        RpcU128<Balance>,
        ExchangeId
    >>>;
}

const RUNTIME_ERROR: i64 = 1;

/// A struct that implements the `ZenlinkDexApi`.
pub struct ZenlinkDex<C, M> {
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> ZenlinkDex<C, M> {
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

impl<C, Block, AccountId, AssetId, TokenBalance, Balance, ExchangeId>
ZenlinkDexApi<<Block as BlockT>::Hash, AccountId, AssetId, TokenBalance, Balance, ExchangeId>
for ZenlinkDex<C, Block>
    where
        Block: BlockT,
        AccountId: Codec,
        AssetId: Codec,
        TokenBalance: Codec + Display + FromStr,
        Balance: Codec + Display + FromStr,
        ExchangeId: Codec,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: ZenlinkDexRuntimeApi<Block, AccountId, AssetId, TokenBalance, Balance, ExchangeId>,
{
    fn get_token_info(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        token_id: AssetId,
    ) -> Result<Option<TokenInfo<
        RpcU128<TokenBalance>
    >>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(
            || self.client.info().best_hash
        ));

        Ok(api.get_token_info(&at, token_id)
            .map(|option| {
                option
                    .map(|token_info| {
                        TokenInfo {
                            current_supply: token_info.current_supply.into(),
                            name: token_info.name,
                            symbol: token_info.symbol,
                            decimals: token_info.decimals,
                        }
                    })
            })
            .map_err(runtime_error_into_rpc_err)?)
    }

    fn get_token_balance(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        token_id: AssetId,
        owner: AccountId,
    ) -> Result<RpcU128<TokenBalance>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(
            || self.client.info().best_hash
        ));

        Ok(api.get_token_balance(&at, token_id, owner)
            .map(|token_balance| token_balance.into())
            .map_err(runtime_error_into_rpc_err)?)
    }

    fn get_token_allowance(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        token_id: AssetId,
        owner: AccountId,
        spender: AccountId,
    ) -> Result<RpcU128<TokenBalance>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(
            || self.client.info().best_hash)
        );

        Ok(api.get_token_allowance(&at, token_id, owner, spender)
            .map(|token_balance| token_balance.into())
            .map_err(runtime_error_into_rpc_err)?)
    }

    fn get_exchange_by_token_id(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        token_id: AssetId,
    ) -> Result<Option<ExchangeInfo<
        AccountId,
        AssetId,
        RpcU128<TokenBalance>,
        RpcU128<Balance>,
        ExchangeId
    >>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(
            || self.client.info().best_hash)
        );

        Ok(api.get_exchange_by_token_id(&at, token_id)
            .map(|option| {
                option
                    .map(|exchange_info| {
                        ExchangeInfo {
                            exchange: Exchange {
                                token_id: exchange_info.exchange.token_id,
                                liquidity_id: exchange_info.exchange.liquidity_id,
                                account: exchange_info.exchange.account,
                            },
                            token_reserve: exchange_info.token_reserve.into(),
                            currency_reserve: exchange_info.currency_reserve.into(),
                            exchange_id: exchange_info.exchange_id,
                        }
                    })
            }).map_err(runtime_error_into_rpc_err)?)
    }

    fn get_exchange_by_id(
        &self,
        at: Option<<Block as BlockT>::Hash>,
        id: ExchangeId,
    ) -> Result<Option<ExchangeInfo<
        AccountId,
        AssetId,
        RpcU128<TokenBalance>,
        RpcU128<Balance>,
        ExchangeId
    >>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(
            || self.client.info().best_hash)
        );

        Ok(api.get_exchange_by_id(&at, id)
            .map(|option| {
                option
                    .map(|exchange_info| {
                        ExchangeInfo {
                            exchange: Exchange {
                                token_id: exchange_info.exchange.token_id,
                                liquidity_id: exchange_info.exchange.liquidity_id,
                                account: exchange_info.exchange.account,
                            },
                            token_reserve: exchange_info.token_reserve.into(),
                            currency_reserve: exchange_info.currency_reserve.into(),
                            exchange_id: exchange_info.exchange_id,
                        }
                    })
            }).map_err(runtime_error_into_rpc_err)?)
    }

    // TODO：Pagination
    fn get_exchanges(
        &self,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<Vec<ExchangeInfo<
        AccountId,
        AssetId,
        RpcU128<TokenBalance>,
        RpcU128<Balance>,
        ExchangeId
    >>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        Ok(api.get_exchanges(&at)
            .map(|exchanges| {
                exchanges
                    .into_iter()
                    .map(|exchange_info| ExchangeInfo {
                        exchange: Exchange {
                            token_id: exchange_info.exchange.token_id,
                            liquidity_id: exchange_info.exchange.liquidity_id,
                            account: exchange_info.exchange.account,
                        },
                        token_reserve: exchange_info.token_reserve.into(),
                        currency_reserve: exchange_info.currency_reserve.into(),
                        exchange_id: exchange_info.exchange_id,
                    })
                    .collect::<Vec<_>>()
            })
            .map_err(runtime_error_into_rpc_err)?)
    }
}

/// Converts a runtime trap into an RPC error.
fn runtime_error_into_rpc_err(err: impl std::fmt::Debug) -> RpcError {
    RpcError {
        code: ErrorCode::ServerError(RUNTIME_ERROR),
        message: "Runtime trapped".into(),
        data: Some(format!("{:?}", err).into()),
    }
}