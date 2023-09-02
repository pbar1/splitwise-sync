use axum::routing::get;
use axum::routing::post;
use axum::Router;
use clap::Args;
use tokio::signal::unix::SignalKind;

use crate::handlers;

#[derive(Debug, Args)]
pub struct ServerArgs {
    /// Address and port to expose the HTTP server on
    #[arg(long, default_value = "0.0.0.0:8080")]
    addr: std::net::SocketAddr,
}

impl ServerArgs {
    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("building routes");
        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/interactions", post(handlers::interactions));

        tracing::info!(addr = %&self.addr, "starting server");
        axum::Server::bind(&self.addr)
            .serve(app.into_make_service())
            .with_graceful_shutdown(handle_signals())
            .await?;

        tracing::info!("exiting");
        Ok(())
    }
}

// NOTE: Signal handling seems to be crucial for running in K8s, as without
// handling SIGTERM the pod gets stuck in the "Terminating" state forever
async fn handle_signals() {
    let mut sigint = tokio::signal::unix::signal(SignalKind::interrupt())
        .expect("unable to create signal handler");
    let mut sigterm = tokio::signal::unix::signal(SignalKind::terminate())
        .expect("unable to create signal handler");

    tokio::select! {
        _ = sigint.recv() => tracing::info!("got sigint"),
        _ = sigterm.recv() => tracing::info!("got sigterm"),
    }
}
