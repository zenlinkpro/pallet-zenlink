#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::unnecessary_mut_passed)]

use codec::Codec;
use sp_std::vec::Vec;

use zenlink_dex::Exchange;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
	pub trait ZenlinkDexApi<AccountId, AssetId>
	where
	AccountId: Codec,
	AssetId: Codec,
	{
		fn exchanges() -> Vec<Exchange<AccountId, AssetId>>;
	}
}