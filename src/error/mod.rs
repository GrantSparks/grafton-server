use std::io;

use {
    crate::axum::{
        body::Body,
        http::{Response as HttpResponse, StatusCode},
        response::{IntoResponse, Response},
    },
    thiserror::Error,
    tokio_rustls::rustls::Error as RustlsError,
    url::ParseError,
};

#[derive(Debug, Error)]
pub enum Error {
    #[error("Configuration error: {0}")]
    ConfigError(#[from] grafton_config::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] io::Error),

    #[error("TLS configuration error: {0}")]
    TlsConfigError(#[from] RustlsError),

    #[error("Error formatting URL with protocol '{protocol}', hostname '{hostname}', port {port}, cause {cause}, inner {inner}")]
    UrlFormatError {
        protocol: String,
        hostname: String,
        port: u16,
        inner: ParseError,
        cause: String,
    },

    #[error("Missing router factory")]
    MissingRouterFactory,
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let (status, error_message) = (
            StatusCode::INTERNAL_SERVER_ERROR,
            "An unexpected error occurred".to_string(),
        );

        let full_message = format!("{status}: {error_message}");
        let body = Body::from(full_message);

        HttpResponse::builder().status(status).body(body).unwrap() // Safe unwrap since we're constructing a valid response
    }
}
