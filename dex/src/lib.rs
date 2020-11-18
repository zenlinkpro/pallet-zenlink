//! # DEX Module
//!
//! ## Overview
//!
//! Built-in decentralized exchange modules in Substrate 2.0 network, the swap
//! mechanism refers to the design of Uniswap V1.

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    Parameter,
    traits::{Currency, ExistenceRequirement, Get},
};
use frame_system::ensure_signed;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{ModuleId, RuntimeDebug};
use sp_runtime::traits::{
    AccountIdConversion, AtLeast32Bit, CheckedAdd, MaybeSerializeDeserialize, Member, One,
    SaturatedConversion, Zero,
};

use zenlink_assets::AssetInfo;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod rpc;

/// ZLK liquidity token info
const ZLK: &AssetInfo = &AssetInfo {
    name: *b"liquidity_zlk_v1",
    /// ZLK
    symbol: [90, 76, 75, 0, 0, 0, 0, 0],
    decimals: 0u8,
};

/// The Dex main structure
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, Default)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub struct Exchange<AccountId, AssetId> {
    // The token being swapped.
    pub token_id: AssetId,
    // The exchange liquidity asset.
    pub liquidity_id: AssetId,
    // This exchange account.
    pub account: AccountId,
}

/// The wrapper of exchangeId and assetId to access
#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug)]
pub enum SwapHandler<ExchangeId, AssetId> {
    ExchangeId(ExchangeId),
    AssetId(AssetId),
}

impl<ExchangeId, AssetId> SwapHandler <ExchangeId, AssetId> {
    pub fn from_exchange_id(id :ExchangeId) -> Self {
        Self::ExchangeId(id)
    }

    pub fn from_asset_id(id :AssetId) -> Self {
        Self::AssetId(id)
    }
}

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

type TokenBalance<T> = <T as zenlink_assets::Trait>::TokenBalance;

type SwapHandlerOf<T> =
    SwapHandler<<T as Trait>::ExchangeId, <T as zenlink_assets::Trait>::AssetId>;

/// The pallet's configuration trait.
pub trait Trait: frame_system::Trait + zenlink_assets::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// The exchange id for every trade pair
    type ExchangeId: Parameter + Member + AtLeast32Bit + Default + Copy + MaybeSerializeDeserialize;
    /// Currency for transfer currencies
    type Currency: Currency<Self::AccountId>;
    /// The dex's module id, used for deriving sovereign account IDs.
    type ModuleId: Get<ModuleId>;
}

decl_storage! {
    trait Store for Module<T: Trait> as DexStorage {
        /// Token to exchange: asset_id -> exchange_id
        TokenToExchange get(fn token_to_exchange): map hasher(opaque_blake2_256) T::AssetId => Option<T::ExchangeId>;
        /// Liquidity to exchange: zlk_asset_id -> exchange_id
        ZLKToExchange get(fn zlk_to_exchange): map hasher(opaque_blake2_256) T::AssetId => Option<T::ExchangeId>;
        /// The exchanges: exchange_id -> exchange
        Exchanges get(fn get_exchange): map hasher(opaque_blake2_256) T::ExchangeId => Option<Exchange<T::AccountId, T::AssetId>>;
        /// The next exchange identifier
        NextExchangeId get(fn next_exchange_id): T::ExchangeId;
    }
}

decl_event! {
    pub enum Event<T> where
       AccountId = <T as frame_system::Trait>::AccountId,
       BalanceOf = BalanceOf<T>,
       Id = <T as Trait>::ExchangeId,
       TokenBalance = <T as zenlink_assets::Trait>::TokenBalance,
    {
        /// An exchange was created. \[ExchangeId, ExchangeAccount\]
        ExchangeCreated(Id, AccountId),
        /// Add liquidity success. \[ExchangeId, ExchangeAccount, Currency_input, Token_input\]
        LiquidityAdded(Id, AccountId, BalanceOf, TokenBalance),
        /// Remove liquidity from the exchange success. \[ExchangeId, ExchangeAccount, Currency_output, Token_output\]
        LiquidityRemoved(Id, AccountId, BalanceOf, TokenBalance),
        /// Use supply token to swap currency. \[ExchangeId, Buyer, Currency_bought, token_sold, Recipient\]
        CurrencyPurchase(Id, AccountId, BalanceOf, TokenBalance, AccountId),
        /// Use supply currency to swap token. \[ExchangeId, Buyer, Currency_sold, Tokens_bought, Recipient\]
        TokenPurchase(Id, AccountId, BalanceOf, TokenBalance, AccountId),
        /// Use supply token to swap other token. \[ExchangeId, Other_ExchangeId, Buyer, token_sold, other_token_bought, Recipient\]
        OtherTokenPurchase(Id, Id, AccountId, TokenBalance, TokenBalance, AccountId),
    }
}

decl_error! {
    /// Error for dex module.
    pub enum Error for Module<T: Trait> {
        /// Denied `ZLK` liquidity token to swap in exchange
        DeniedSwap,
        /// Deadline hit.
        Deadline,
        /// Token not exists at this AssetId.
        TokenNotExists,
        /// Zero token supplied.
        ZeroToken,
        /// Zero currency supplied.
        ZeroCurrency,
        /// Exchange not exists at this Id.
        ExchangeNotExists,
        /// A Exchange already exists for a particular AssetId.
        ExchangeAlreadyExists,
        /// Requested zero liquidity.
        RequestedZeroLiquidity,
        /// Would add too many token to liquidity.
        TooManyToken,
        /// Not enough liquidity created.
        TooLowLiquidity,
        /// Trying to burn zero shares.
        BurnZeroZLKShares,
        /// No liquidity in the exchange.
        NoLiquidity,
        /// Not enough currency will be returned.
        NotEnoughCurrency,
        /// Not enough token will be returned.
        NotEnoughToken,
        /// Exchange would cost too much in currency.
        TooExpensiveCurrency,
        /// Exchange would cost too much in token.
        TooExpensiveToken,
        /// The allowance token balance of exchange spend too low.
        AllowanceLow
    }
}

// TODO: weight
// TODO: transaction
// TODO: exchange fee
// The pallet's dispatched functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Create an exchange with the token which would swap with native currency
        ///
        /// - `token_id`: The exist asset's id.
        #[weight = 0]
        pub fn create_exchange(origin,
            token_id: T::AssetId,
        ) -> dispatch::DispatchResult
        {
            let asset_info = <zenlink_assets::Module<T>>::asset_info(&token_id);
            ensure!(asset_info.is_some(), Error::<T>::TokenNotExists);
            ensure!(Self::zlk_to_exchange(token_id).is_none(), Error::<T>::DeniedSwap);
            ensure!(Self::token_to_exchange(token_id).is_none(), Error::<T>::ExchangeAlreadyExists);

            let exchange_id = Self::next_exchange_id();
            let next_id = exchange_id.checked_add(&One::one())
                .ok_or("Overflow")?;

            let account: T::AccountId = T::ModuleId::get().into_sub_account(exchange_id);

            // create a new lp token for exchange
            let liquidity_id = <zenlink_assets::Module<T>>::inner_issue(&account, Zero::zero(), ZLK);
            let new_exchange = Exchange {
                token_id,
                liquidity_id,
                account: account.clone(),
            };

            <TokenToExchange<T>>::insert(token_id, exchange_id);
            <ZLKToExchange<T>>::insert(liquidity_id, exchange_id);
            <Exchanges<T>>::insert(exchange_id, new_exchange);
            <NextExchangeId<T>>::put(next_id);

            Self::deposit_event(RawEvent::ExchangeCreated(exchange_id, account));

            Ok(())
        }

        /// Injecting liquidity to specific exchange liquidity pool in the form of depositing
        /// currencies to the exchange account and issue liquidity pool token in proportion
        /// to the caller who is the liquidity provider.
        /// The liquidity pool token, shares `ZLK`, allowed to transfer but can't swap in exchange
        /// it represents the proportion of assets in liquidity pool.
        ///
        /// - `swap_handler`: The wrapper of exchangeId and assetId to access.
        /// - `currency_amount`: Amount of base currency to lock.
        /// - `min_liquidity`: Min amount of exchange shares(ZLK) to create.
        /// - `max_token`: Max amount of token to input.
        /// - `deadline`: When to invalidate the transaction.
        #[weight = 0]
        pub fn add_liquidity(origin,
            swap_handler: SwapHandlerOf<T>,
            currency_amount: BalanceOf<T>,
            min_liquidity: TokenBalance<T>,
            max_token: TokenBalance<T>,
            deadline: T::BlockNumber,
        ) -> dispatch::DispatchResult
        {
            // Deadline is to prevent front-running (more of a problem on Ethereum).
            let now = frame_system::Module::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            let who = ensure_signed(origin)?;

            ensure!(max_token > Zero::zero(), Error::<T>::ZeroToken);
            ensure!(currency_amount > Zero::zero(), Error::<T>::ZeroCurrency);

            let exchange_id = Self::get_exchange_id(&swap_handler)?;

            if let Some(exchange) = Self::get_exchange(exchange_id) {
                let total_liquidity = <zenlink_assets::Module<T>>::total_supply(&exchange.liquidity_id);

                if total_liquidity > Zero::zero() {
                    ensure!(min_liquidity > Zero::zero(), Error::<T>::RequestedZeroLiquidity);
                    let currency_reserve = Self::convert(Self::get_currency_reserve(&exchange));
                    let token_reserve = Self::get_token_reserve(&exchange);
                    let token_amount = Self::convert(currency_amount) * token_reserve / currency_reserve;
                    let liquidity_minted = Self::convert(currency_amount) * total_liquidity / currency_reserve;

                    ensure!(max_token >= token_amount, Error::<T>::TooManyToken);
                    ensure!(liquidity_minted >= min_liquidity, Error::<T>::TooLowLiquidity);
                    ensure!(<zenlink_assets::Module<T>>::allowances(&exchange.token_id, &who, &exchange.account) >= token_amount, Error::<T>::AllowanceLow);

                    T::Currency::transfer(&who, &exchange.account, currency_amount, ExistenceRequirement::KeepAlive)?;
                    <zenlink_assets::Module<T>>::inner_mint(&exchange.liquidity_id, &who, liquidity_minted)?;
                    <zenlink_assets::Module<T>>::inner_transfer_from(&exchange.token_id, &who, &exchange.account, &exchange.account, token_amount)?;

                    Self::deposit_event(RawEvent::LiquidityAdded(exchange_id, who, currency_amount, token_amount));
                } else {
                    // Fresh exchange with no liquidity
                    let token_amount = max_token;
                    ensure!(<zenlink_assets::Module<T>>::allowances(&exchange.token_id, &who, &exchange.account) >= token_amount, Error::<T>::AllowanceLow);

                    T::Currency::transfer(&who, &exchange.account, currency_amount, ExistenceRequirement::KeepAlive)?;

                    let initial_liquidity: u64 = T::Currency::free_balance(&exchange.account).saturated_into::<u64>();

                    <zenlink_assets::Module<T>>::inner_mint(&exchange.liquidity_id, &who, initial_liquidity.saturated_into())?;
                    <zenlink_assets::Module<T>>::inner_transfer_from(&exchange.token_id, &who, &exchange.account, &exchange.account, token_amount)?;

                    Self::deposit_event(RawEvent::LiquidityAdded(exchange_id, who, currency_amount, token_amount));
                }

                Ok(())
            } else {
                Err(Error::<T>::ExchangeNotExists.into())
            }
        }

        /// Remove liquidity from specific exchange liquidity pool in the form of burning
        /// shares(ZLK), and withdrawing currencies from the exchange account in proportion,
        /// and withdraw liquidity incentive interest.
        ///
        /// - `swap_handler`: The wrapper of exchangeId and assetId to access.
        /// - `zlk_to_burn`: Liquidity amount to remove.
        /// - `min_currency`: Minimum currency to withdraw.
        /// - `min_token`: Minimum token to withdraw.
        /// - `deadline`: When to invalidate the transaction.
        #[weight = 0]
        pub fn remove_liquidity(origin,
            swap_handler: SwapHandlerOf<T>,
            zlk_to_burn: TokenBalance<T>,
            min_currency: BalanceOf<T>,
            min_token: TokenBalance<T>,
            deadline: T::BlockNumber,
        ) -> dispatch::DispatchResult
        {
            let now = frame_system::Module::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            let who = ensure_signed(origin)?;

            ensure!(zlk_to_burn > Zero::zero(), Error::<T>::BurnZeroZLKShares);

            let exchange_id = Self::get_exchange_id(&swap_handler)?;

            if let Some(exchange) = Self::get_exchange(exchange_id) {
                let total_liquidity = <zenlink_assets::Module<T>>::total_supply(&exchange.liquidity_id);

                ensure!(total_liquidity > Zero::zero(), Error::<T>::NoLiquidity);

                let token_reserve = Self::get_token_reserve(&exchange);
                let currency_reserve = Self::get_currency_reserve(&exchange);
                let currency_amount = zlk_to_burn * Self::convert(currency_reserve) / total_liquidity;
                let token_amount = zlk_to_burn * token_reserve / total_liquidity;

                ensure!(Self::unconvert(currency_amount) >= min_currency, Error::<T>::NotEnoughCurrency);
                ensure!(token_amount >= min_token, Error::<T>::NotEnoughToken);

                <zenlink_assets::Module<T>>::inner_burn(&exchange.liquidity_id, &who, zlk_to_burn)?;
                T::Currency::transfer(&exchange.account, &who, Self::unconvert(currency_amount), ExistenceRequirement::AllowDeath)?;
                <zenlink_assets::Module<T>>::inner_transfer(&exchange.token_id, &exchange.account, &who, token_amount)?;

                Self::deposit_event(RawEvent::LiquidityRemoved(exchange_id, who, Self::unconvert(currency_amount), token_amount));

                Ok(())
            } else {
                Err(Error::<T>::ExchangeNotExists.into())
            }
        }

        /// Swap currency to token.
        ///
        /// User specifies the exact amount of currency to sold and the amount not less the minimum
        /// token to be returned.
        /// - `swap_handler`: The wrapper of exchangeId and assetId to access.
        /// - `currency_sold`: The balance amount to be sold.
        /// - `min_token`: The minimum token expected to buy.
        /// - `deadline`: When to invalidate the transaction.
        /// - `recipient`: Receiver of the bought token.
        #[weight = 0]
        pub fn currency_to_token_input(origin,
            swap_handler: SwapHandlerOf<T>,
            currency_sold: BalanceOf<T>,
            min_token: TokenBalance<T>,
            deadline: T::BlockNumber,
            recipient: T::AccountId,
        ) -> dispatch::DispatchResult
        {
            let now = frame_system::Module::<T>::block_number();
            ensure!(deadline > now, Error::<T>::Deadline);

            let buyer = ensure_signed(origin)?;

            ensure!(currency_sold > Zero::zero(), Error::<T>::ZeroCurrency);
            ensure!(min_token > Zero::zero(), Error::<T>::ZeroToken);

            let exchange_id = Self::get_exchange_id(&swap_handler)?;

            if let Some(exchange) = Self::get_exchange(exchange_id) {
                let token_reserve = Self::get_token_reserve(&exchange);
                let currency_reserve = Self::get_currency_reserve(&exchange);
                let tokens_bought = Self::get_input_price(Self::convert(currency_sold), Self::convert(currency_reserve), token_reserve);

                ensure!(tokens_bought >= min_token, Error::<T>::NotEnoughToken);

                T::Currency::transfer(&buyer, &exchange.account, currency_sold, ExistenceRequirement::KeepAlive)?;
                <zenlink_assets::Module<T>>::inner_transfer(&exchange.token_id, &exchange.account, &recipient, tokens_bought)?;

                Self::deposit_event(RawEvent::TokenPurchase(exchange_id, buyer, currency_sold, tokens_bought, recipient));

                Ok(())
            } else {
                Err(Error::<T>::ExchangeNotExists.into())
            }
        }

        /// Swap currency to token.
        ///
        /// User specifies the maximum currency to be sold and the exact amount of
        /// token to be returned.
        /// - `swap_handler`: The wrapper of exchangeId and assetId to access.
        /// - `tokens_bought`: The amount of the token to buy.
        /// - `max_currency`: The maximum currency expected to be sold.
        /// - `deadline`: When to invalidate the transaction.
        /// - `recipient`: Receiver of the bought token.
        #[weight = 0]
        pub fn currency_to_token_output(origin,
            swap_handler: SwapHandlerOf<T>,
            tokens_bought: TokenBalance<T>,
            max_currency: BalanceOf<T>,
            deadline: T::BlockNumber,
            recipient: T::AccountId,
        ) -> dispatch::DispatchResult
        {
            let now = frame_system::Module::<T>::block_number();
            ensure!(deadline >= now, Error::<T>::Deadline);

            let buyer = ensure_signed(origin)?;

            ensure!(tokens_bought > Zero::zero(), Error::<T>::ZeroToken);
            ensure!(max_currency > Zero::zero(), Error::<T>::ZeroCurrency);

            let exchange_id = Self::get_exchange_id(&swap_handler)?;

            if let Some(exchange) = Self::get_exchange(exchange_id) {
                let token_reserve = Self::get_token_reserve(&exchange);
                let currency_reserve = Self::get_currency_reserve(&exchange);
                let currency_sold = Self::get_output_price(tokens_bought, Self::convert(currency_reserve), token_reserve);

                ensure!(Self::unconvert(currency_sold) <= max_currency, Error::<T>::TooExpensiveCurrency);

                T::Currency::transfer(&buyer, &exchange.account, Self::unconvert(currency_sold), ExistenceRequirement::KeepAlive)?;
                <zenlink_assets::Module<T>>::inner_transfer(&exchange.token_id, &exchange.account, &recipient, tokens_bought)?;

                Self::deposit_event(RawEvent::TokenPurchase(exchange_id, buyer, Self::unconvert(currency_sold), tokens_bought, recipient));

                Ok(())
            } else {
                Err(Error::<T>::ExchangeNotExists.into())
            }
        }

        /// Swap token to currency.
        ///
        /// User specifies the exact amount of token to sold and the amount not less the minimum
        /// currency to be returned.
        /// - `swap_handler`: The wrapper of exchangeId and assetId to access.
        /// - `token_sold`: The token balance amount to be sold.
        /// - `min_currency`: The minimum currency expected to buy.
        /// - `deadline`: When to invalidate the transaction.
        /// - `recipient`: Receiver of the bought currency.
        #[weight = 0]
        pub fn token_to_currency_input(origin,
            swap_handler: SwapHandlerOf<T>,
            token_sold: TokenBalance<T>,
            min_currency: BalanceOf<T>,
            deadline: T:: BlockNumber,
            recipient: T::AccountId,
        ) -> dispatch::DispatchResult
        {
            let now = frame_system::Module::<T>::block_number();
            ensure!(deadline >= now, Error::<T>::Deadline);

            let buyer = ensure_signed(origin)?;

            ensure!(token_sold > Zero::zero(), Error::<T>::ZeroToken);
            ensure!(min_currency > Zero::zero(), Error::<T>::ZeroCurrency);

            let exchange_id = Self::get_exchange_id(&swap_handler)?;

            if let Some(exchange) = Self::get_exchange(exchange_id) {
                let token_reserve = Self::get_token_reserve(&exchange);
                let currency_reserve = Self::get_currency_reserve(&exchange);
                let currency_bought = Self::get_input_price(token_sold, token_reserve, Self::convert(currency_reserve));

                ensure!(currency_bought >= Self::convert(min_currency), Error::<T>::NotEnoughCurrency);
                ensure!(<zenlink_assets::Module<T>>::allowances(&exchange.token_id, &buyer, &exchange.account) >= token_sold, Error::<T>::AllowanceLow);

                T::Currency::transfer(&exchange.account, &recipient, Self::unconvert(currency_bought), ExistenceRequirement::AllowDeath)?;
                <zenlink_assets::Module<T>>::inner_transfer_from(&exchange.token_id, &buyer, &exchange.account, &exchange.account, token_sold)?;

                Self::deposit_event(RawEvent::CurrencyPurchase(exchange_id, buyer, Self::unconvert(currency_bought), token_sold, recipient));

                Ok(())
            } else {
                Err(Error::<T>::ExchangeNotExists.into())
            }
        }

        /// Swap token to currency.
        ///
        /// User specifies the maximum token to be sold and the exact
        /// currency to be returned.
        /// - `swap_handler`: The wrapper of exchangeId and assetId to access.
        /// - `currency_bought`: The balance of currency to buy.
        /// - `max_token`: The maximum currency expected to be sold.
        /// - `deadline`: When to invalidate the transaction.
        /// - `recipient`: Receiver of the bought currency.
        #[weight = 0]
        pub fn token_to_currency_output(origin,
            swap_handler: SwapHandlerOf<T>,
            currency_bought: BalanceOf<T>,
            max_token: TokenBalance<T>,
            deadline: T::BlockNumber,
            recipient: T::AccountId,
        ) -> dispatch::DispatchResult
        {
            let now = frame_system::Module::<T>::block_number();
            ensure!(deadline >= now, Error::<T>::Deadline);

            let buyer = ensure_signed(origin)?;

            ensure!(max_token > Zero::zero(), Error::<T>::ZeroToken);
            ensure!(currency_bought > Zero::zero(), Error::<T>::ZeroCurrency);

            let exchange_id = Self::get_exchange_id(&swap_handler)?;

            if let Some(exchange) = Self::get_exchange(exchange_id) {
                let token_reserve = Self::get_token_reserve(&exchange);
                let currency_reserve = Self::get_currency_reserve(&exchange);
                let token_sold = Self::get_output_price(Self::convert(currency_bought), token_reserve, Self::convert(currency_reserve));

                ensure!(max_token >= token_sold, Error::<T>::TooExpensiveToken);
                ensure!(<zenlink_assets::Module<T>>::allowances(&exchange.token_id, &buyer, &exchange.account) >= token_sold, Error::<T>::AllowanceLow);

                T::Currency::transfer(&exchange.account, &buyer, currency_bought, ExistenceRequirement::AllowDeath)?;
                <zenlink_assets::Module<T>>::inner_transfer_from(&exchange.token_id, &buyer, &exchange.account, &exchange.account, token_sold)?;

                Self::deposit_event(RawEvent::CurrencyPurchase(exchange_id, buyer, currency_bought, token_sold, recipient));

                Ok(())
            } else {
                Err(Error::<T>::ExchangeNotExists.into())
            }
        }

        /// Swap token to other token.
        ///
        /// User specifies the exact amount of token to sold and the amount not less the minimum
        /// other token to be returned.
        /// - `swap_handler`: The wrapper of exchangeId and assetId to access.
        /// - `other_swap_handle`: The wrapper of exchangeId and assetId to access.
        /// - `token_sold`: The token balance amount to be sold.
        /// - `min_other_token`: The minimum other token expected to buy.
        /// - `deadline`: When to invalidate the transaction.
        /// - `recipient`: Receiver of the bought other token.
        #[weight = 0]
        pub fn token_to_token_input(origin,
            swap_handler: SwapHandlerOf<T>,
            other_swap_handle: SwapHandlerOf<T>,
            token_sold: TokenBalance<T>,
            min_other_token: TokenBalance<T>,
            deadline: T::BlockNumber,
            recipient: T::AccountId,
        ) -> dispatch::DispatchResult
        {
            let now = frame_system::Module::<T>::block_number();
            ensure!(deadline >= now, Error::<T>::Deadline);

            let buyer = ensure_signed(origin)?;

            ensure!(token_sold > Zero::zero(), Error::<T>::ZeroToken);
            ensure!(min_other_token > Zero::zero(), Error::<T>::ZeroToken);

            let exchange_id = Self::get_exchange_id(&swap_handler)?;
            let other_exchange_id = Self::get_exchange_id(&other_swap_handle)?;
            let get_exchange = Self::get_exchange(exchange_id);
            let get_othere_exchange = Self::get_exchange(other_exchange_id);
            if get_exchange.is_none() || get_othere_exchange.is_none() {
                return Err(Error::<T>::ExchangeNotExists.into())
            }
            let exchange = get_exchange.unwrap();
            let other_exchange = get_othere_exchange.unwrap();

            let token_reserve = Self::get_token_reserve(&exchange);
            let currency_reserve = Self::get_currency_reserve(&exchange);
            let currency_bought = Self::get_input_price(token_sold, token_reserve, Self::convert(currency_reserve));

            let other_token_reserve = Self::get_token_reserve(&other_exchange);
            let other_currency_reserve = Self::get_currency_reserve(&other_exchange);
            let other_token_bought = Self::get_input_price(currency_bought, Self::convert(other_currency_reserve), other_token_reserve);

            ensure!(other_token_bought >= min_other_token, Error::<T>::NotEnoughToken);
            ensure!(<zenlink_assets::Module<T>>::allowances(&exchange.token_id, &buyer, &exchange.account) >= token_sold, Error::<T>::AllowanceLow);

            <zenlink_assets::Module<T>>::inner_transfer_from(&exchange.token_id, &buyer, &exchange.account, &exchange.account, token_sold)?;
            T::Currency::transfer(&exchange.account, &other_exchange.account, Self::unconvert(currency_bought), ExistenceRequirement::KeepAlive)?;
            <zenlink_assets::Module<T>>::inner_transfer(&other_exchange.token_id, &other_exchange.account, &recipient, other_token_bought)?;

            Self::deposit_event(RawEvent::OtherTokenPurchase(exchange_id, other_exchange_id, buyer, token_sold, other_token_bought, recipient));

            Ok(())
        }

        /// Swap token to other token.
        ///
        /// User specifies the maximum token to be sold and the exact
        /// other token to be returned.
        /// - `swap_handler`: The wrapper of exchangeId and assetId to access.
        /// - `other_swap_handle`: The wrapper of exchangeId and assetId to access..
        /// - `other_token_bought`: The amount of the other token to buy.
        /// - `max_token`: The maximum token expected to be sold.
        /// - `deadline`: When to invalidate the transaction.
        /// - `recipient`: Receiver of the bought currency.
        #[weight = 0]
        pub fn token_to_token_output(origin,
            swap_handler: SwapHandlerOf<T>,
            other_swap_handle: SwapHandlerOf<T>,
            other_token_bought: TokenBalance<T>,
            max_token: TokenBalance<T>,
            deadline: T::BlockNumber,
            recipient: T::AccountId,
        ) -> dispatch::DispatchResult  {
            let now = frame_system::Module::<T>::block_number();
            ensure!(deadline >= now, Error::<T>::Deadline);

            let buyer = ensure_signed(origin)?;

            ensure!(other_token_bought > Zero::zero(), Error::<T>::ZeroToken);
            ensure!(max_token > Zero::zero(), Error::<T>::ZeroToken);

            let exchange_id = Self::get_exchange_id(&swap_handler)?;
            let other_exchange_id = Self::get_exchange_id(&other_swap_handle)?;
            let get_exchange = Self::get_exchange(exchange_id);
            let get_othere_exchange = Self::get_exchange(other_exchange_id);
            if get_exchange.is_none() || get_othere_exchange.is_none() {
                return Err(Error::<T>::ExchangeNotExists.into())
            }
            let exchange = get_exchange.unwrap();
            let other_exchange = get_othere_exchange.unwrap();

            let other_token_reserve = Self::get_token_reserve(&other_exchange);
            let other_currency_reserve = Self::get_currency_reserve(&other_exchange);
            let currency_sold = Self::get_output_price(other_token_bought, Self::convert(other_currency_reserve), other_token_reserve);

            let token_reserve = Self::get_token_reserve(&exchange);
            let currency_reserve = Self::get_currency_reserve(&exchange);
            let token_sold = Self::get_output_price(currency_sold, token_reserve, Self::convert(currency_reserve));

            ensure!(max_token >= token_sold, Error::<T>::TooExpensiveToken);
            ensure!(<zenlink_assets::Module<T>>::allowances(&exchange.token_id, &buyer, &exchange.account) >= token_sold, Error::<T>::AllowanceLow);

            <zenlink_assets::Module<T>>::inner_transfer_from(&exchange.token_id, &buyer, &exchange.account, &exchange.account, token_sold)?;
            T::Currency::transfer(&exchange.account, &other_exchange.account, Self::unconvert(currency_sold), ExistenceRequirement::KeepAlive)?;
            <zenlink_assets::Module<T>>::inner_transfer(&other_exchange.token_id, &other_exchange.account, &recipient, other_token_bought)?;

            Self::deposit_event(RawEvent::OtherTokenPurchase(exchange_id, other_exchange_id, buyer, token_sold, other_token_bought, recipient));

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Get the exchange_id by unwrapping the swap_handler.
    /// Return exist exchange_id or `ExchangeNotExists` error.
    pub fn get_exchange_id(swap_handler: &SwapHandlerOf<T>) -> Result<T::ExchangeId, Error<T>> {
        match swap_handler {
            SwapHandler::ExchangeId(exchange_id) => Ok(*exchange_id),
            SwapHandler::AssetId(asset_id) => {
                Self::token_to_exchange(asset_id).ok_or(Error::<T>::ExchangeNotExists)
            }
        }
    }

    pub fn get_exchange_info(id :T::ExchangeId) -> Option<Exchange<T::AccountId, T::AssetId>> {
        Self::get_exchange(id)
    }
    /// Swap Currency to Token.
    /// Return Amount of Token bought.
    pub fn get_currency_to_token_input_price(
        exchange: &Exchange<T::AccountId, T::AssetId>,
        currency_sold: BalanceOf<T>,
    ) -> TokenBalance<T> {
        if currency_sold == Zero::zero() {
            return Zero::zero();
        }

        let token_reserve = Self::get_token_reserve(exchange);
        let currency_reserve = Self::get_currency_reserve(exchange);
        Self::get_input_price(
            Self::convert(currency_sold),
            Self::convert(currency_reserve),
            token_reserve,
        )
    }

    /// Swap Currency to Token.
    /// Return Amount of Currency sold.
    pub fn get_currency_to_token_output_price(
        exchange: &Exchange<T::AccountId, T::AssetId>,
        tokens_bought: TokenBalance<T>,
    ) -> TokenBalance<T> {
        if tokens_bought == Zero::zero() {
            return Zero::zero();
        }

        let token_reserve = Self::get_token_reserve(exchange);
        let currency_reserve = Self::get_currency_reserve(exchange);
        Self::get_output_price(
            tokens_bought,
            Self::convert(currency_reserve),
            token_reserve,
        )
    }

    /// Swap Token to Currency.
    /// Return Amount of Currency bought.
    pub fn get_token_to_currency_input_price(
        exchange: &Exchange<T::AccountId, T::AssetId>,
        token_sold: TokenBalance<T>,
    ) -> TokenBalance<T> {
        if token_sold == Zero::zero() {
            return Zero::zero();
        }

        let token_reserve = Self::get_token_reserve(exchange);
        let currency_reserve = Self::get_currency_reserve(exchange);
        Self::get_input_price(token_sold, token_reserve, Self::convert(currency_reserve))
    }

    /// Swap Token to Currency.
    /// Return Amount of Token bought.
    pub fn get_token_to_currency_output_price(
        exchange: &Exchange<T::AccountId, T::AssetId>,
        currency_bought: BalanceOf<T>,
    ) -> TokenBalance<T> {
        if currency_bought == Zero::zero() {
            return Zero::zero();
        }

        let token_reserve = Self::get_token_reserve(exchange);
        let currency_reserve = Self::get_currency_reserve(exchange);
        Self::get_output_price(
            Self::convert(currency_bought),
            token_reserve,
            Self::convert(currency_reserve),
        )
    }

    /// Pricing function for converting between Currency and Token.
    /// Return Amount of Currency or Token bought.
    fn get_input_price(
        input_amount: TokenBalance<T>,
        input_reserve: TokenBalance<T>,
        output_reserve: TokenBalance<T>,
    ) -> TokenBalance<T> {
        let input_amount_with_fee = input_amount * 997.into();
        let numerator = input_amount_with_fee * output_reserve;
        let denominator = (input_reserve * 1000.into()) + input_amount_with_fee;
        numerator / denominator
    }

    /// Pricing function for converting between Currency and Token.
    /// Return Amount of Currency or Token sold.
    fn get_output_price(
        output_amount: TokenBalance<T>,
        input_reserve: TokenBalance<T>,
        output_reserve: TokenBalance<T>,
    ) -> TokenBalance<T> {
        let numerator = input_reserve * output_amount * 1000.into();
        let denominator = (output_reserve - output_amount) * 997.into();
        numerator / denominator + 1.into()
    }

    /// Convert BalanceOf to TokenBalance
    /// e.g. BalanceOf is u128, TokenBalance is u64
    fn convert(balance_of: BalanceOf<T>) -> TokenBalance<T> {
        let m = balance_of.saturated_into::<u64>();
        m.saturated_into()
    }

    /// Convert TokenBalance to BalanceOf
    /// e.g. BalanceOf is u128, TokenBalance is u64
    fn unconvert(token_balance: TokenBalance<T>) -> BalanceOf<T> {
        let m = token_balance.saturated_into::<u64>();
        m.saturated_into()
    }

    /// Get the token balance of the exchange liquidity pool
    fn get_token_reserve(exchange: &Exchange<T::AccountId, T::AssetId>) -> TokenBalance<T> {
        <zenlink_assets::Module<T>>::balance_of(&exchange.token_id, &exchange.account)
    }

    /// Get the currency balance of the exchange liquidity pool
    fn get_currency_reserve(exchange: &Exchange<T::AccountId, T::AssetId>) -> BalanceOf<T> {
        T::Currency::free_balance(&exchange.account)
    }
}
