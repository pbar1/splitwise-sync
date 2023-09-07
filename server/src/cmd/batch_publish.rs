use std::path::PathBuf;

use anyhow::Context;
use clap::Args;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;

use crate::models::mint::Transaction;

#[derive(Debug, Args)]
pub struct BatchPublishArgs {
    /// Glob pattern to use for searching for the latest two transaction files
    /// to diff against each other
    #[arg(long, short = 'g', default_value = "transactions.*.json*")]
    glob: String,

    /// Output file containing only new transactions
    #[arg(long, default_value = "new_transactions.json")]
    output: String,

    /// ID of the Discord channel to publish messages to
    #[arg(long, env = "DISCORD_CHANNEL_ID")]
    channel_id: Id<ChannelMarker>,
}

impl BatchPublishArgs {
    pub async fn run(&self, token: String) -> anyhow::Result<()> {
        let _client = twilight_http::Client::new(token);

        // Find the last two files via lexical sort. This assumes that the transaction
        // files are named by timestamp
        let mut files: Vec<PathBuf> = glob::glob(&self.glob)?.flat_map(|x| x).collect();
        files.sort();
        let cur = files.pop().context("no files found via glob")?;
        let prev = files.pop().context("only one file found via glob")?;
        let cur = cur.to_string_lossy();
        let prev = prev.to_string_lossy();

        // Compute the new transactions that only appear in the latest file
        anti_join(&cur, &prev, &self.output)?;
        let data = std::fs::read(&self.output)?;
        let txns: Vec<Transaction> = serde_json::from_slice(&data)?;

        for txn in txns {
            let id = &txn.id;
            let date = &txn.date;
            let description = &txn.description;
            let amount = &txn.amount;
            tracing::debug!(%id, %date, %description, %amount, "found new transaction");
        }

        Ok(())
    }
}

fn anti_join(cur: &str, prev: &str, output: &str) -> anyhow::Result<()> {
    let query = format!(
        r"
        CREATE TABLE before AS SELECT * FROM '{prev}';
        CREATE TABLE after AS SELECT * FROM '{cur}';
        COPY (
            SELECT after.*
            FROM after
            ANTI JOIN before
            ON after.id = before.id
        ) TO '{output}' (ARRAY true);
        "
    );

    // Shells out to the DuckDB CLI. When using the Rust library, was getting an
    // error like so with the same query:
    // ```
    // Error: Catalog Error: Table with name transactions.1693517401.json.gz does not exist!
    // ```
    std::process::Command::new("duckdb")
        .args(["-c", &query])
        .status()?;

    Ok(())
}
