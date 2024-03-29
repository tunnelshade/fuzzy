use std::error::Error;

use tracing_subscriber::{
    self,
    fmt,
    registry::Registry,
    layer::{Layer, SubscriberExt},
    EnvFilter
};

use crate::models;

pub mod network_layer;

pub enum TraceEvent {
    NewEvent(models::NewTraceEvent)
}

pub struct Tracer {
    verbose_n: u64
}

/// Heavy code dedup, fix this when implementing db tracing layer
impl Tracer {
    pub fn new(verbose_n: u64) -> Self {
        Self { verbose_n }
    }

    pub fn set_global_with_layer<L>(self, layer: L) -> Result<(), Box<dyn Error>>
    where
        L: Layer<Registry> + Send + Sync + Sized
    {
        let fmt_layer = fmt::layer()
            .with_target(true);

        let env_filter = EnvFilter::from_default_env()
            .add_directive(match self.verbose_n {
                1 => "fuzzy=info",
                2 => "fuzzy=debug",
                3 => "fuzzy=trace",
                _ => "fuzzy=warn",
            }.parse()?);

        let subscriber = Registry::default()
            .with(layer)
            .with(env_filter)
            .with(fmt_layer);

        tracing::subscriber::set_global_default(subscriber)?;
        Ok(())
    }

    pub fn set_global(self) -> Result<(), Box<dyn Error>> {
        let fmt_layer = fmt::layer()
            .with_target(true);

        let env_filter = EnvFilter::from_default_env()
            .add_directive(match self.verbose_n {
                1 => "fuzzy=info",
                2 => "fuzzy=debug",
                3 => "fuzzy=trace",
                _ => "fuzzy=warn",
            }.parse()?);

        let subscriber = Registry::default()
            .with(env_filter)
            .with(fmt_layer);

        tracing::subscriber::set_global_default(subscriber)?;
        Ok(())
    }
}
