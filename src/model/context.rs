use std::{
    fmt::{Debug, Formatter},
    sync::Arc,
};

use crate::{axum::extract::FromRef, Config, ServerConfigProvider};

#[derive(Clone)]
pub struct Context<C>
where
    C: ServerConfigProvider,
{
    pub config: Arc<C>,
}

impl<C> Debug for Context<C>
where
    C: ServerConfigProvider,
{
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        f.debug_struct("Context")
            .field("config", &self.config)
            .finish()
    }
}

impl<C> Context<C>
where
    C: ServerConfigProvider,
{
    #[must_use]
    pub fn new(config: C) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

impl<C> FromRef<Arc<Context<C>>> for Config
where
    C: ServerConfigProvider,
{
    fn from_ref(state: &Arc<Context<C>>) -> Self {
        state.config.get_server_config().clone()
    }
}
