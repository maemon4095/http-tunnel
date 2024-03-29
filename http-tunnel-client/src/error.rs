#[derive(Debug, thiserror::Error)]
#[error("uri must have scheme")]
pub struct UriMustHasSchemeError;

#[derive(Debug, thiserror::Error)]
#[error("{} tunnel is not supported", self.0)]
pub struct ConnectionSchemeNotSupportedError(pub http::uri::Scheme);

#[derive(Debug, thiserror::Error)]
#[error("use {} connection as base is not supported", self.0)]
pub struct BaseSchemeNotSupportedError(pub http::uri::Scheme);
