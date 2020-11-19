#![allow(clippy::type_complexity)]

#[cfg(feature = "std")]
use std::result::Result as StdResult;

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{de, Deserialize, ser, Serialize};
use sp_std::convert::TryInto;
use sp_std::vec::Vec;

use super::*;

/// Text serialization/deserialization
#[cfg(feature = "std")]
pub mod serde_text {
    use super::*;

    /// A serializer that encodes the bytes as a string
    pub fn serialize<T, S>(value: &T, serializer: S) -> StdResult<S::Ok, S::Error>
        where
            S: ser::Serializer,
            T: AsRef<[u8]>,
    {
        let output = String::from_utf8_lossy(value.as_ref());
        serializer.serialize_str(&output)
    }

    /// A deserializer that decodes the string to the bytes (Vec<u8>)
    pub fn deserialize<'de, D>(deserializer: D) -> StdResult<Vec<u8>, D::Error>
        where
            D: de::Deserializer<'de>,
    {
        let data = String::deserialize(deserializer)?;
        Ok(data.into_bytes())
    }
}

#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct ExchangeInfo<AccountId, AssetId, TokenBalance, Balance, ExchangeId> {
    #[cfg_attr(feature = "std", serde(flatten))]
    pub exchange: Exchange<AccountId, AssetId>,
    pub token_reserve: TokenBalance,
    pub currency_reserve: Balance,
    pub exchange_id: ExchangeId,
}

#[derive(PartialEq, Eq, Clone, Default, Encode, Decode, RuntimeDebug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct TokenInfo<TokenBalance> {
    pub current_supply: TokenBalance,
    #[cfg_attr(feature = "std", serde(with = "self::serde_text"))]
    pub name: Vec<u8>,
    #[cfg_attr(feature = "std", serde(with = "self::serde_text"))]
    pub symbol: Vec<u8>,
    pub decimals: u8,
}

impl<T: Trait> Module<T> {
    pub fn get_token_info(token_id: T::AssetId) -> Option<TokenInfo<T::TokenBalance>> {
        <zenlink_assets::Module<T>>::asset_info(&token_id)
            .map(|asset_info| {
                let current_supply = <zenlink_assets::Module<T>>::total_supply(&token_id);
                TokenInfo {
                    name: asset_info.name
                        .to_vec()
                        .into_iter()
                        .take_while(|c| *c != 0u8)
                        .collect::<Vec<_>>(),
                    symbol: asset_info.symbol
                        .to_vec()
                        .into_iter()
                        .take_while(|c| *c != 0u8)
                        .collect::<Vec<_>>(),
                    decimals: asset_info.decimals,
                    current_supply,
                }
            })
    }
    pub fn get_token_balance(token_id: T::AssetId, owner: T::AccountId) -> TokenBalance<T> {
        <zenlink_assets::Module<T>>::balance_of(&token_id, &owner)
    }

    pub fn get_token_allowance(token_id: T::AssetId, owner: T::AccountId, spender: T::AccountId) -> TokenBalance<T> {
        <zenlink_assets::Module<T>>::allowances(&token_id, &owner, &spender)
    }

    pub fn get_exchange_by_token_id(token_id: T::AssetId) -> Option<ExchangeInfo<T::AccountId, T::AssetId, TokenBalance<T>, BalanceOf<T>, T::ExchangeId>> {
        Self::token_to_exchange(token_id).and_then(|exchange_id| {
            Self::get_exchange_by_id(exchange_id)
        })
    }

    pub fn get_exchange_by_id(exchange_id: T::ExchangeId) -> Option<ExchangeInfo<T::AccountId, T::AssetId, TokenBalance<T>, BalanceOf<T>, T::ExchangeId>> {
        Self::get_exchange_info(exchange_id).
            map(|exchange| {
                let token_reserve = Self::get_token_reserve(&exchange);
                let currency_reserve = Self::get_currency_reserve(&exchange);

                ExchangeInfo {
                    exchange,
                    token_reserve,
                    currency_reserve,
                    exchange_id,
                }
            })
    }

    // TODO：Pagination
    pub fn get_exchanges() -> Vec<ExchangeInfo<T::AccountId, T::AssetId, TokenBalance<T>, BalanceOf<T>, T::ExchangeId>> {
        let exchange_count = Self::next_exchange_id().try_into().unwrap_or_default();

        let mut exchanges = Vec::with_capacity(exchange_count);
        for exchange_id in 0..exchange_count {
            if let Some(exchange) = Self::get_exchange_info((exchange_id as u32).into()) {
                let token_reserve = Self::get_token_reserve(&exchange);
                let currency_reserve = Self::get_currency_reserve(&exchange);
                exchanges.push(ExchangeInfo {
                    exchange,
                    token_reserve,
                    currency_reserve,
                    exchange_id: (exchange_id as u32).into(),
                })
            }
        }

        exchanges
    }
}

#[cfg(test)]
mod rpc_tests {
    use frame_support::assert_ok;

    use crate::mock::*;

    use super::*;

    const ALICE: u128 = 1;
    const EXCHANGE_ACCOUNT: u128 = 15310315390164549602772283245;
    const TEST_TOKEN: &AssetInfo = &AssetInfo {
        name: *b"zenlinktesttoken",
        symbol: *b"TEST____",
        decimals: 0u8,
    };

    #[test]
    fn rpc_get_exchange_by_token_id_should_work() {
        new_test_ext().execute_with(|| {
            assert_eq!(TokenModule::inner_issue(&ALICE, 10000, TEST_TOKEN), 0);
            assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));

            assert!(DexModule::get_exchange_by_token_id(0).is_some());
        });
    }

    #[test]
    fn rpc_get_exchanges_should_work() {
        new_test_ext().execute_with(|| {
            assert_eq!(TokenModule::inner_issue(&ALICE, 10000, TEST_TOKEN), 0);
            assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));

            assert!(!DexModule::get_exchanges().is_empty());
            assert_eq!(DexModule::get_exchanges()[0],
                       ExchangeInfo {
                           exchange: Exchange {
                               token_id: 0,
                               liquidity_id: 1,
                               account: 15310315390164549602772283245,
                           },
                           token_reserve: 0,
                           currency_reserve: 0,
                           exchange_id: 0,
                       }
            );

            // Alice approve 1000 token for EXCHANGE_ACCOUNT
            assert_ok!(TokenModule::inner_approve(
                &0,
                &ALICE,
                &EXCHANGE_ACCOUNT,
                1000
            ));

            // Add 1000 currency and 100 token
            assert_ok!(DexModule::add_liquidity(
                Origin::signed(ALICE),
                SwapHandler::from_exchange_id(0),
                100,
                0,
                1000,
                100
            ));

            assert_eq!(DexModule::get_exchanges()[0],
                       ExchangeInfo {
                           exchange: Exchange {
                               token_id: 0,
                               liquidity_id: 1,
                               account: 15310315390164549602772283245,
                           },
                           token_reserve: 1000,
                           currency_reserve: 100,
                           exchange_id: 0,
                       }
            );
        });
    }
}