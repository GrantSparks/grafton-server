use grafton_config::load_config_from_dir;
use grafton_server::{add, axum::Router, GraftonRouter, Builder, Context, Error, Logger};
use tokio::signal;
use tracing::info;

use std::sync::Arc;

use {
    derivative::Derivative,
    grafton_config::TokenExpandingConfig,
    grafton_server::{
        axum::extract::FromRef, axum::routing::get, Config as ServerConfig, ServerConfigProvider,
    },
    serde::{Deserialize, Serialize},
};

type AppContext = Context<Config>;
type AppRouter = GraftonRouter<Config>;

#[derive(Debug, Serialize, Deserialize, Derivative, Clone)]
#[derivative(Default)]
#[serde(default)]
pub struct CalculatorConfig {
    #[derivative(Default(value = "1"))]
    pub first_number: i32,
    #[derivative(Default(value = "2"))]
    pub second_number: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(flatten)]
    pub base: ServerConfig,
    pub calculator_config: CalculatorConfig,
}

impl ServerConfigProvider for Config {
    fn get_server_config(&self) -> &ServerConfig {
        &self.base
    }
}

impl TokenExpandingConfig for Config {}

impl FromRef<Arc<AppContext>> for CalculatorConfig {
    fn from_ref(state: &Arc<AppContext>) -> Self {
        state.config.calculator_config.clone()
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let config: Config = load_config_from_dir("examples/config")?;

    let _logger_guard = Logger::from_config(&config.base);

    let builder = Builder::new(config);

    let server = builder.with_router(build_todos_router).build()?;

    server.start();
    info!("Server started successfully");

    signal::ctrl_c().await?;
    info!("Server shutdown gracefully");

    Ok(())
}

pub fn build_todos_router(_app_ctx: &Arc<AppContext>) -> AppRouter {
    Router::new().route("/", get(self::get::route_handler))
}
mod get {

    use super::{add, CalculatorConfig};

    use grafton_server::axum::{
        extract::State,
        response::{IntoResponse, Json},
    };

    pub async fn route_handler(
        State(calculator_config): State<CalculatorConfig>,
    ) -> impl IntoResponse {
        // Use the add function to add two numbers from the calculator config
        let sum = add(
            calculator_config.first_number,
            calculator_config.second_number,
        );

        let todos = vec![
            String::from("Collect underpants"),
            String::from("..."),
            format!("Profit! {}", sum),
        ];
        Json(todos).into_response()
    }
}
