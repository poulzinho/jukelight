//! Jukelight's composition root: configuration, wiring and lifecycle.

pub mod config;

use std::sync::Arc;

use jukelight_adapter_artwork::ArtworkProxy;
use jukelight_adapter_bluos::{BluosConfig, BluosMonitor};
use jukelight_adapter_fake::FakeDevice;
use jukelight_adapter_http::{HttpConfig, router_with_shutdown};
use jukelight_adapter_music_assistant::{MusicAssistantProvider, UrlError};
use jukelight_application::{
    Clock, DeviceMonitor, DisplayController, PlayerMonitor, SessionProvider, SystemClock,
};
use jukelight_domain::DisplayPolicy;

use crate::config::{Config, DeviceConfig};

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error(transparent)]
    Scenario(#[from] jukelight_adapter_fake::UnknownScenario),
    #[error("MA_URL: {0}")]
    MusicAssistantUrl(#[from] UrlError),
    #[error("cannot serve HTTP: {0}")]
    Io(#[from] std::io::Error),
}

/// Runs Jukelight until the process receives Ctrl-C or SIGTERM.
pub async fn run(config: Config) -> Result<(), RunError> {
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);

    // The fake device's controls must live as long as the device.
    let (device, _fake_controls): (Arc<dyn DeviceMonitor>, _) = match &config.device {
        DeviceConfig::Bluos { host, port } => {
            tracing::info!("following a BluOS player");
            let config = BluosConfig {
                host: host.clone(),
                port: *port,
            };
            (
                Arc::new(BluosMonitor::new(&config, Arc::clone(&clock))),
                None,
            )
        }
        DeviceConfig::Fake { scenario } => {
            tracing::info!(scenario, "using the fake device");
            let (device, controls) = FakeDevice::new(scenario, Arc::clone(&clock))?;
            (Arc::new(device), Some(controls))
        }
    };

    let mut providers: Vec<Arc<dyn SessionProvider>> = Vec::new();
    if let Some(config) = &config.music_assistant {
        tracing::info!("following Music Assistant sessions");
        let provider = MusicAssistantProvider::new(config, Arc::clone(&clock))?;
        tokio::spawn(provider.run());
        providers.push(Arc::new(provider));
    }

    let monitor = PlayerMonitor::new(providers);
    let player = monitor.handle();
    let display = DisplayController::new(
        DisplayPolicy::new(config.display_grace).with_input_limit(config.input_display_limit),
        Arc::clone(&clock),
    );
    let display_handle = display.handle();
    tokio::spawn(display.run(player.subscribe_published()));
    tokio::spawn(monitor.run(device));

    let (stop, shutdown) = tokio::sync::watch::channel(false);
    let app = router_with_shutdown(
        player,
        display_handle,
        Arc::new(ArtworkProxy::new()),
        &HttpConfig::default(),
        shutdown,
    );
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.http_port)).await?;
    tracing::info!(port = config.http_port, "serving the kiosk and API");
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown_signal().await;
            tracing::info!("stopping");
            // End the kiosks' live streams; they reconnect to the next start.
            let _ = stop.send(true);
        })
        .await?;
    tracing::info!("stopped");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        let _ = tokio::signal::ctrl_c().await;
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
