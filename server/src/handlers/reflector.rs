use axum::http::StatusCode;
use once_cell::sync::Lazy;
use twilight_model::channel::message::component::ActionRow;
use twilight_model::channel::message::component::Button;
use twilight_model::channel::message::component::ButtonStyle;
use twilight_model::channel::message::component::Component;

static DISCORD_BOT_TOKEN: Lazy<String> =
    Lazy::new(|| std::env::var("DISCORD_BOT_TOKEN").expect("DISCORD_BOT_TOKEN required"));
static DISCORD_CHANNEL_ID: Lazy<u64> = Lazy::new(|| {
    std::env::var("DISCORD_CHANNEL_ID")
        .expect("DISCORD_CHANNEL_ID required")
        .parse::<u64>()
        .expect("error parsing string to u64")
});

pub async fn reflector() -> Result<(), StatusCode> {
    let client = twilight_http::Client::new(DISCORD_BOT_TOKEN.to_string());

    let channel_id = twilight_model::id::Id::new(*DISCORD_CHANNEL_ID);

    let button = Component::ActionRow(ActionRow {
        components: Vec::from([Component::Button(Button {
            custom_id: Some("click_one".to_owned()),
            disabled: false,
            emoji: None,
            label: Some("Click me!".to_owned()),
            style: ButtonStyle::Primary,
            url: None,
        })]),
    });

    let message = client
        .create_message(channel_id)
        .content("this is some text")
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .components(&[button])
        .map_err(|_| StatusCode::BAD_REQUEST)?
        .tts(true)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let status = message.status();
    tracing::info!(?status, "discord client received response code");

    Ok(())
}
