// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, ensure, Parameter};
use frame_system::ensure_signed;
use sp_runtime::{DispatchResult, RuntimeDebug};
use sp_runtime::traits::{
    AtLeast32Bit, AtLeast32BitUnsigned, CheckedSub, MaybeSerializeDeserialize, Member, One, Saturating, StaticLookup,
    Zero,
};

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

/// the symbol of asset.
type Symbol = [u8; 8];
/// the name of asset.
type Name = [u8; 16];

#[derive(Encode, Decode, Eq, PartialEq, Clone, RuntimeDebug, Default)]
pub struct AssetInfo {
    pub name: Name,
    pub symbol: Symbol,
    pub decimals: u8,
}

/// The module configuration trait.
pub trait Trait: frame_system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;

    /// The units in which we record balances.
    type TokenBalance: Member + Parameter + AtLeast32BitUnsigned + Default + Copy + MaybeSerializeDeserialize;

    /// The arithmetic type of asset identifier.
    type AssetId: Parameter + AtLeast32Bit + Default + Copy + MaybeSerializeDeserialize;
}

// TODO: weight
// TODO: transaction
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;
        /// Issue a new class of pallet-zenlink assets. There are, and will only ever be, `total`
        /// such assets and they'll all belong to the `origin` initially. It will have an
        /// identifier `AssetId` instance: this will be specified in the `Issued` event.
        ///
        /// - `total`: initial total supply.
        /// - `asset_info`: the asset info contains `name`, `symbol`, `decimals`.
        #[weight = 0]
        fn issue(origin, #[compact] total: T::TokenBalance, asset_info: AssetInfo) {
            let origin = ensure_signed(origin)?;
            Self::inner_issue(&origin, total, &asset_info);
        }

        /// Move some assets from one holder to another.
        ///
        /// - `id`: the asset id.
        /// - `target`: the receiver of the asset.
        /// - `amount`: the amount of the asset to transfer.
        #[weight = 0]
        fn transfer(origin,
            #[compact] id: T::AssetId,
            target: <T::Lookup as StaticLookup>::Source,
            #[compact] amount: T::TokenBalance
        ) {
            let origin = ensure_signed(origin)?;
            let target = T::Lookup::lookup(target)?;

            Self::inner_transfer(&id, &origin, &target, amount)?;
        }

        /// Allow spender to withdraw from the origin account
        ///
        /// - `id`: the asset id.
        /// - `spender`: the spender account.
        /// - `amount`: the amount of allowance.
        #[weight = 0]
        fn approve(origin,
            #[compact] id: T::AssetId,
            spender: <T::Lookup as StaticLookup>::Source,
            #[compact] amount: T::TokenBalance
        ) {
            let owner = ensure_signed(origin)?;
            let spender = T::Lookup::lookup(spender)?;

            Self::inner_approve(&id, &owner, &spender, amount)?;
        }

        /// Send amount of asset from Account `from` to Account `target`.
        ///
        /// - `id`: the asset id.
        /// - `from`: the source of the asset to be transferred.
        /// - `target`: the receiver of the asset to be transferred.
        /// - `amount`: the amount of asset to be transferred.
        #[weight = 0]
        fn transfer_from(origin,
            #[compact] id: T::AssetId,
            from: <T::Lookup as StaticLookup>::Source,
            target: <T::Lookup as StaticLookup>::Source,
            #[compact] amount: T::TokenBalance
        ){
            let spender = ensure_signed(origin)?;
            let owner = T::Lookup::lookup(from)?;
            let target = T::Lookup::lookup(target)?;

            Self::inner_transfer_from(&id, &owner, &spender, &target, amount)?;
        }
    }
}

decl_event! {
    pub enum Event<T> where
        <T as frame_system::Trait>::AccountId,
        <T as Trait>::TokenBalance,
        <T as Trait>::AssetId,
    {
        /// Some assets were issued. \[asset_id, owner, initial_supply\]
        Issued(AssetId, AccountId, TokenBalance),
        /// Some assets were transferred. \[asset_id, owner, target, amount\]
        Transferred(AssetId, AccountId, AccountId, TokenBalance),
        /// Some assets were allowable. \[asset_id, owner, spender, amount\]
        Approval(AssetId, AccountId, AccountId, TokenBalance),

        /// other module generated. e.g. dex.

        /// Some assets were burned. \[asset_id, owner, amount\]
        Burned(AssetId, AccountId, TokenBalance),
        /// Some assets were minted. \[asset_id, owner, amount\]
        Minted(AssetId, AccountId, TokenBalance),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Transfer amount should be non-zero.
        AmountZero,
        /// Account balance must be greater than or equal to the transfer amount.
        BalanceLow,
        /// Balance should be non-zero.
        BalanceZero,
        /// Account allowance balance must be greater than or equal to the transfer_from amount.
        AllowanceLow,
        /// Asset has not been created.
        AssetNotExists,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Assets {
        /// The info of the asset by any given asset id.
        AssetInfos: map hasher(twox_64_concat) T::AssetId => Option<AssetInfo>;
        /// The number of units of assets held by any given account.
        Balances: map hasher(blake2_128_concat) (T::AssetId, T::AccountId) => T::TokenBalance;
        /// The next asset identifier up for grabs.
        NextAssetId get(fn next_asset_id): T::AssetId;
        /// The total unit supply of an asset.
        ///
        /// TWOX-NOTE: `AssetId` is trusted, so this is safe.
        TotalSupply: map hasher(twox_64_concat) T::AssetId => T::TokenBalance;
        /// The allowance of assets held by spender who can spend from owner.
        Allowances: map hasher(blake2_128_concat) (T::AssetId, T::AccountId, T::AccountId) => T::TokenBalance;
    }
}

// The main implementation block for the module.
impl<T: Trait> Module<T> {
    /// public mutable functions

    /// Implement of the issue function.
    ///
    /// Return the asset id.
    pub fn inner_issue(
        owner: &T::AccountId,
        initial_supply: T::TokenBalance,
        info: &AssetInfo,
    ) -> T::AssetId {
        let id = Self::next_asset_id();
        <NextAssetId<T>>::mutate(|id| *id += One::one());

        <Balances<T>>::insert((id, owner), initial_supply);
        <TotalSupply<T>>::insert(id, initial_supply);
        <AssetInfos<T>>::insert(id, info);

        Self::deposit_event(RawEvent::Issued(id, owner.clone(), initial_supply));

        id
    }

    /// Implement of the transfer function.
    pub fn inner_transfer(
        id: &T::AssetId,
        owner: &T::AccountId,
        target: &T::AccountId,
        amount: T::TokenBalance,
    ) -> DispatchResult {
        let owner_balance = <Balances<T>>::get((id, owner));
        ensure!(!amount.is_zero(), Error::<T>::AmountZero);
        ensure!(owner_balance >= amount, Error::<T>::BalanceLow);

        let new_balance = owner_balance.saturating_sub(amount);

        <Balances<T>>::mutate((id, owner), |balance| *balance = new_balance);
        <Balances<T>>::mutate((id, target), |balance| {
            *balance = balance.saturating_add(amount)
        });

        Self::deposit_event(RawEvent::Transferred(
            *id,
            owner.clone(),
            target.clone(),
            amount,
        ));

        Ok(())
    }

    /// Implement of the approve function.
    pub fn inner_approve(
        id: &T::AssetId,
        owner: &T::AccountId,
        spender: &T::AccountId,
        amount: T::TokenBalance,
    ) -> DispatchResult {
        <Allowances<T>>::mutate((id, owner, spender), |balance| *balance = amount);

        Self::deposit_event(RawEvent::Approval(
            *id,
            owner.clone(),
            spender.clone(),
            amount,
        ));

        Ok(())
    }

    /// Implement of the transfer_from function.
    pub fn inner_transfer_from(
        id: &T::AssetId,
        owner: &T::AccountId,
        spender: &T::AccountId,
        target: &T::AccountId,
        amount: T::TokenBalance,
    ) -> DispatchResult {
        let allowance = <Allowances<T>>::get((id, owner, spender));
        let new_balance = allowance
            .checked_sub(&amount)
            .ok_or(Error::<T>::AllowanceLow)?;

        Self::inner_transfer(&id, &owner, &target, amount)?;

        <Allowances<T>>::mutate((id, owner, spender), |balance| *balance = new_balance);

        Ok(())
    }

    /// Increase the total supply of the asset
    pub fn inner_mint(id: &T::AssetId, owner: &T::AccountId, amount: T::TokenBalance) -> DispatchResult {
        ensure!(Self::asset_info(id).is_some(), Error::<T>::AssetNotExists);

        let new_balance = <Balances<T>>::get((id, owner)).saturating_add(amount);

        <Balances<T>>::mutate((id, owner), |balance| *balance = new_balance);
        <TotalSupply<T>>::mutate(id, |supply| {
            *supply = supply.saturating_add(amount);
        });

        Self::deposit_event(RawEvent::Minted(*id, owner.clone(), amount));

        Ok(())
    }

    /// Decrease the total supply of the asset
    pub fn inner_burn(id: &T::AssetId, owner: &T::AccountId, amount: T::TokenBalance) -> DispatchResult {
        ensure!(Self::asset_info(id).is_some(), Error::<T>::AssetNotExists);

        let new_balance = <Balances<T>>::get((id, owner))
            .checked_sub(&amount)
            .ok_or(Error::<T>::BalanceLow)?;

        <Balances<T>>::mutate((id, owner), |balance| *balance = new_balance);
        <TotalSupply<T>>::mutate(id, |supply| {
            *supply = supply.saturating_sub(amount);
        });

        Self::deposit_event(RawEvent::Burned(*id, owner.clone(), amount));

        Ok(())
    }

    // Public immutable functions

    /// Get the asset `id` balance of `owner`.
    pub fn balance_of(id: &T::AssetId, owner: &T::AccountId) -> T::TokenBalance {
        <Balances<T>>::get((id, owner))
    }

    /// Get the total supply of an asset `id`.
    pub fn total_supply(id: &T::AssetId) -> T::TokenBalance {
        <TotalSupply<T>>::get(id)
    }

    /// Get the allowance balance of the spender under owner
    pub fn allowances(id: &T::AssetId, owner: &T::AccountId, spender: &T::AccountId) -> T::TokenBalance {
        <Allowances<T>>::get((id, owner, spender))
    }

    /// Get the info of the asset by th asset `id`
    pub fn asset_info(id: &T::AssetId) -> Option<AssetInfo> {
        <AssetInfos<T>>::get(id)
    }
}