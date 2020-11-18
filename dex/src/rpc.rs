use sp_std::convert::TryInto;
use sp_std::vec::Vec;

use super::*;

impl<T: Trait> Module<T> {
    pub fn exchanges() -> Vec<Exchange<T::AccountId, T::AssetId>> {
        let exchange_count = Self::next_exchange_id().try_into().unwrap_or_default();

        let mut exchanges = Vec::with_capacity(exchange_count);
        for exchange_id in 0..exchange_count {
            if let Some(exchange_info) = Self::get_exchange_info((exchange_id as u32).into()) {
                exchanges.push(exchange_info)
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
    const TEST_TOKEN: &AssetInfo = &AssetInfo {
        name: *b"zenlinktesttoken",
        symbol: *b"TEST____",
        decimals: 0u8,
    };

    #[test]
    fn rpc_should_work() {
        new_test_ext().execute_with(|| {
            assert_eq!(TokenModule::inner_issue(&ALICE, 10000, TEST_TOKEN), 0);
            assert_ok!(DexModule::create_exchange(Origin::signed(ALICE), 0));
            assert!(!DexModule::exchanges().is_empty());

            assert_eq!(DexModule::exchanges()[0],
                       Exchange {
                           token_id: 0,
                           liquidity_id: 1,
                           account: 15310315390164549602772283245,
                       });
        });
    }
}