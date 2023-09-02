use axum::routing::get;
use axum::routing::post;
use axum::Router;
use clap::Args;
use ed25519_compact::PublicKey;
use tokio::signal::unix::SignalKind;

use crate::handlers;

#[derive(Debug, Args)]
pub struct ServerArgs {
    /// Address and port to expose the HTTP server on
    #[arg(long, default_value = "0.0.0.0:8080")]
    addr: std::net::SocketAddr,

    /// Discord application public key
    #[arg(long, env = "DISCORD_PUBLIC_KEY")]
    public_key: String,
}

#[derive(Clone)]
pub struct ServerState {
    pub public_key: PublicKey,
    pub bot_token: String,
}

impl ServerArgs {
    pub async fn run(&self, token: String) -> anyhow::Result<()> {
        let public_key = hex::decode(&self.public_key)?;
        let public_key = PublicKey::from_slice(&public_key)?;

        let state = ServerState {
            public_key,
            bot_token: token,
        };

        tracing::info!("building routes");
        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/interactions", post(handlers::interactions))
            .with_state(state);

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
