use anyhow::Result;
use axum::Router;
use serde::Deserialize;
use std::{net::SocketAddr, sync::OnceLock};
use time_tz::{Tz, timezones::get_by_name};

#[derive(Debug, Deserialize)]
pub struct GeneralConfig {
    pub listen: String,
    pub timezone: String,
}

static TIMEZONE: OnceLock<&Tz> = OnceLock::new();

pub async fn init_timezone(config: &GeneralConfig) -> Result<()> {
    let timezone = get_by_name(&config.timezone)
        .ok_or_else(|| anyhow::anyhow!("Invalid timezone configuration: {}", config.timezone))?;
    TIMEZONE
        .set(timezone)
        .map_err(|_| anyhow::anyhow!("Failed to set OnceLock<&Tz>"))
}

pub fn timezone() -> &'static Tz {
    TIMEZONE.get().expect("OnceLock<&Tz> not initialized")
}

pub async fn serve(config: &GeneralConfig, router: Router) -> Result<()> {
    let listener = tokio::net::TcpListener::bind(&config.listen).await?;
    tracing::debug!("listening on {}", listener.local_addr()?);
    axum::serve(
        listener,
        router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await?;
    Ok(())
}
