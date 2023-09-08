#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::similar_names)]

pub mod cmd;
pub mod handlers;
pub mod models;

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
    command: Command,
}

/// Global flags that will be flattened into the CLI and available to all
/// subcommands
#[derive(Debug, Args)]
struct GlobalArgs {
    /// Log level
    #[arg(long, env = "RUST_LOG", default_value = "info")]
    log_level: String,

    /// Token to authenticate with Discord
    #[arg(long, env = "DISCORD_BOT_TOKEN")]
    bot_token: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    init_tracing(&args.global_args.log_level)?;
    tracing::debug!("finished init");

    let token = args.global_args.bot_token;

    match args.command {
        Command::Server(args) => args.run(token).await?,
        Command::Publish(args) => args.run(token).await?,
        Command::BatchPublish(args) => args.run(token).await?,
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
