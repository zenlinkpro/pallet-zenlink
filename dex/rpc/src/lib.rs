//! RPC interface for the transaction payment module.

use std::sync::Arc;

use codec::Codec;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};

use zenlink_dex::Exchange;
use zenlink_dex_runtime_api::ZenlinkDexApi as ZenlinkDexRuntimeApi;

#[rpc]
pub trait ZenlinkDexApi<BlockHash, AccountId, AssetId> {
    #[rpc(name = "zenlinkDex_exchanges")]
    fn exchanges(&self, at: Option<BlockHash>) -> Result<Vec<Exchange<AccountId, AssetId>>>;
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

impl<C, Block, AccountId, AssetId> ZenlinkDexApi<<Block as BlockT>::Hash, AccountId, AssetId> for ZenlinkDex<C, Block>
    where
        Block: BlockT,
        AccountId: Codec,
        AssetId: Codec,
        C: Send + Sync + 'static,
        C: ProvideRuntimeApi<Block>,
        C: HeaderBackend<Block>,
        C::Api: ZenlinkDexRuntimeApi<Block, AccountId, AssetId>,
{
    fn exchanges(&self, at: Option<<Block as BlockT>::Hash>) -> Result<Vec<Exchange<AccountId, AssetId>>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(|| self.client.info().best_hash));

        Ok(api.exchanges(&at).map_err(|e| runtime_error_into_rpc_err(e))?)
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