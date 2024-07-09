use std::sync::Arc;

use crate::{model::Context, tracing::debug, Error, GraftonRouter, RouterFactory, ServerConfigProvider};

use super::server::Server;

pub struct Builder<C>
where
    C: ServerConfigProvider,
{
    app_ctx: Arc<Context<C>>,
    router_factory: Option<Box<RouterFactory<C>>>,
}

impl<C> Builder<C>
where
    C: ServerConfigProvider,
{
    /// # Errors
    ///
    /// This function will return an error if the config is invalid
    pub fn new(config: C) -> Self {
        debug!("Initializing ServerBuilder with config: {:?}", config);

        let context = { Context::new(config) };

        let context = Arc::new(context);

        Self {
            app_ctx: context,
            router_factory: None,
        }
    }

    #[must_use]
    pub fn with_router<F>(mut self, factory: F) -> Self
    where
        F: FnOnce(&Arc<Context<C>>) -> GraftonRouter<C> + Send + 'static,
    {
        self.router_factory = Some(Box::new(factory));
        self
    }

    /// Build the server.
    ///
    /// # Errors
    ///
    /// This function will return an error if the config is invalid
    pub fn build(self) -> Result<Server<C>, Error> {
        let app_ctx = self.app_ctx;

        let router = self
            .router_factory
            .ok_or(Error::MissingRouterFactory)
            .map(|factory| factory(&app_ctx))?;

        Ok(Server {
            router: router.with_state(app_ctx.clone()),
            config: app_ctx.config.clone(),
        })
    }
}
