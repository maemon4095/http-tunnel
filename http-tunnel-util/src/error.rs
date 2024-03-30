#[derive(Debug, thiserror::Error)]
#[error("uri must have scheme")]
pub struct UriMustHasSchemeError;

#[derive(Debug, thiserror::Error)]
#[error("use {} connection as base is not supported", self.0)]
pub struct BaseSchemeNotSupportedError(pub http::uri::Scheme);

pub type BoxError = Box<dyn std::error::Error + std::marker::Send + Sync + 'static>;
