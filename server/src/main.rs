#![warn(clippy::pedantic)]

mod cmd;
mod discord;
mod handlers;

use clap::Args;
use clap::Parser;

use crate::cmd::Command;

/// Splitwise sync utility
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[clap(flatten)]
    global_args: GlobalArgs,

    #[clap(subcommand)]
    command: cmd::Command,
}

/// Global flags that will be flattened into the CLI and available to all
/// subcommands
#[derive(Debug, Args)]
struct GlobalArgs {
    /// Log level
    #[arg(long, env = "RUST_LOG", default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    init_tracing(&args.global_args.log_level)?;
    tracing::debug!("finished init");

    match args.command {
        Command::Server(args) => args.run().await?,
        Command::Publish(args) => args.run().await?,
    }

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
