//! Jukelight's composition root: configuration, wiring and lifecycle.

pub mod config;

use std::sync::Arc;

use jukelight_adapter_artwork::ArtworkProxy;
use jukelight_adapter_bluos::{BluosConfig, BluosMonitor};
use jukelight_adapter_fake::FakeDevice;
use jukelight_adapter_http::{HttpConfig, router};
use jukelight_application::{Clock, DeviceMonitor, PlayerMonitor, SystemClock};

use crate::config::{Config, DeviceConfig};

#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error(transparent)]
    Scenario(#[from] jukelight_adapter_fake::UnknownScenario),
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
            (Arc::new(BluosMonitor::new(&config, clock)), None)
        }
        DeviceConfig::Fake { scenario } => {
            tracing::info!(scenario, "using the fake device");
            let (device, controls) = FakeDevice::new(scenario, clock)?;
            (Arc::new(device), Some(controls))
        }
    };

    let monitor = PlayerMonitor::new(Vec::new());
    let player = monitor.handle();
    tokio::spawn(monitor.run(device));

    let app = router(
        player,
        Arc::new(ArtworkProxy::new()),
        &HttpConfig::default(),
    );
    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.http_port)).await?;
    tracing::info!(port = config.http_port, "serving the kiosk and API");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
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
