use crate::adapter::adapters::jev::JevAdapter;
use crate::adapter::{Adapter, AdapterCtx};
use crate::{AdapterKind, Request, Response, Result};

/// Static dispatch: run one request through the adapter for `kind`.
pub(crate) async fn exec(kind: AdapterKind, ctx: AdapterCtx<'_>, request: &Request) -> Result<Response> {
	match kind {
		AdapterKind::Jev => JevAdapter.exec(ctx, request).await,
	}
}
