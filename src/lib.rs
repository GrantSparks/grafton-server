#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![forbid(unsafe_code)]

mod error;
pub mod model;
mod util;

mod core;
pub use core::add; // TODO:  Trivial example of a public function

use std::sync::Arc;

use grafton_config::TokenExpandingConfig;
use serde::{Deserialize, Serialize};
use strum::{EnumString, VariantNames};

pub use {
    axum,
    core::{builder::Builder, server::Server},
    error::Error,
    model::Context,
    tracing,
    util::{Config, Logger, SslConfig},
};

pub type GraftonRouter<C> = crate::axum::Router<Arc<Context<C>>>;

pub type RouterFactory<C> = dyn FnOnce(&Arc<Context<C>>) -> GraftonRouter<C> + Send + 'static;

pub trait ServerConfigProvider: TokenExpandingConfig {
    fn get_server_config(&self) -> &Config;
}

#[derive(
    Default, EnumString, VariantNames, Debug, Serialize, Deserialize, Clone, PartialEq, Eq,
)]
#[strum(serialize_all = "snake_case")]
pub enum Verbosity {
    Trace,
    #[default]
    Info,
    Debug,
    Warn,
    Error,
}
