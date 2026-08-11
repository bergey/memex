use anyhow::Result;
use std::time::Instant;
use tracing_subscriber::{filter::EnvFilter, fmt, prelude::*};

pub fn init() -> Result<()> {
    if std::env::var("LOG_FORMAT") == Ok("pretty".to_string()) {
        tracing_subscriber::registry()
            .with(fmt::layer())
            .with(EnvFilter::from_default_env())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(fmt::layer().json().flatten_event(true))
            .with(EnvFilter::from_default_env())
            .init();
    }

    Ok(())
}

pub fn hist_time_since(hist: &prometheus::Histogram, start: Instant) {
    let elapsed = Instant::now() - start;
    hist.observe(elapsed.as_secs_f64());
}
