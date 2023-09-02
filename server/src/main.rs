#![warn(clippy::pedantic)]

mod discord;
mod handlers;

use axum::routing::get;
use axum::routing::post;
use axum::Router;
use clap::Parser;
use tokio::signal::unix::SignalKind;

/// Simple web server
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Address and port to expose the HTTP server on
    #[arg(long, default_value = "0.0.0.0:8080")]
    addr: std::net::SocketAddr,

    /// Log level
    #[arg(long, env = "RUST_LOG", default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    init_tracing(&args.log_level)?;
    tracing::debug!("finished init");

    tracing::info!("building routes");
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/interactions", post(handlers::interactions))
        .route("/msg", get(handlers::reflector));

    tracing::info!(addr = %&args.addr, "starting server");
    axum::Server::bind(&args.addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(handle_signals())
        .await?;

    tracing::info!("exiting");
    Ok(())
}

fn init_tracing(log_level: &str) -> anyhow::Result<()> {
    let env_filter = tracing_subscriber::EnvFilter::try_new(log_level)?;

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
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
