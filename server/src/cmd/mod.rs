mod publish;
mod server;

pub use self::publish::PublishArgs;
pub use self::server::ServerArgs;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Run a Discord webhook server to listen for interactions
    Server(ServerArgs),

    /// Publish transactions as messages to a Discord channel
    Publish(PublishArgs),
}
