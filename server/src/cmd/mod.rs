pub mod batch_publish;
pub mod publish;
pub mod server;

#[derive(Debug, clap::Subcommand)]
pub enum Command {
    /// Run a Discord webhook server to listen for interactions
    Server(server::ServerArgs),

    /// Publish transactions as messages to a Discord channel
    Publish(publish::PublishArgs),

    /// Batch publish new transactions found from the diff of two Mint
    /// transaction JSON files
    BatchPublish(batch_publish::BatchPublishArgs),
}
