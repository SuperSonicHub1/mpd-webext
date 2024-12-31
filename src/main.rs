mod mpd;
mod yt_dlp;
mod routes;
mod server_util;
mod job_manager;
mod config; 

use std::{sync::Arc, time::Duration};

use axum::{
    routing::get,
    Extension, Router,
};
use config::SERVER_ADDR;
use job_manager::JobManager;
use mpd::Mpd;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;
use tower_sessions::{MemoryStore, SessionManagerLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Tracing support
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer().without_time())
        .init();

    let mpd = Arc::new(Mpd::new().await?);
    yt_dlp::create_download_dir(&mpd).await?;

    let job_manager = Arc::new(JobManager::new(mpd.clone()));
    let jb1 = job_manager.clone();
    let job_manager_task = tokio::spawn(async move {
        jb1.clone().run().await;
    });

    let session_store = MemoryStore::default();
    let session_layer = SessionManagerLayer::new(session_store).with_secure(false);

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/toggle-playback", get(routes::toggle_playback))
        .route("/download", get(routes::download))
        .route("/queue", get(routes::queue))
        .layer(Extension(mpd))
        .layer(Extension(job_manager))
        // Flash messages
        .layer(axum_messages::MessagesManagerLayer)
        .layer(session_layer)
        .layer(TraceLayer::new_for_http())
        // Graceful shutdown will wait for outstanding requests to complete.
        // Add a timeout so requests don't hang forever.
        .layer(TimeoutLayer::new(Duration::from_secs(10)));

    let addr = SERVER_ADDR;
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Serving on {addr}");
    axum::serve(listener, app)
        .with_graceful_shutdown(server_util::shutdown_signal())
        .await?;

    job_manager_task.await?;

    Ok(())
}
