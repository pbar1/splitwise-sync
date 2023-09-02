pub fn get_client() -> anyhow::Result<twilight_http::Client> {
    let token = std::env::var("DISCORD_BOT_TOKEN")?;

    let client = twilight_http::Client::new(token);

    Ok(client)
}
