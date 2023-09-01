use axum::body::Bytes;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::Json;
use ed25519_compact::PublicKey;
use ed25519_compact::Signature;
use once_cell::sync::Lazy;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::InteractionType::Ping;
use twilight_model::http::interaction::InteractionResponse;
use twilight_model::http::interaction::InteractionResponseType;

const HEADER_SIGNATURE: &str = "X-Signature-Ed25519";
const HEADER_TIMESTAMP: &str = "X-Signature-Timestamp";

static PUBLIC_KEY: Lazy<PublicKey> = Lazy::new(|| {
    let key_hex = std::env::var("DISCORD_PUBLIC_KEY").expect("DISCORD_PUBLIC_KEY required");
    let key_bytes = hex::decode(key_hex).expect("error decoding hex");
    PublicKey::from_slice(&key_bytes).expect("error decoding ed25519 public key")
});

pub async fn interactions(
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<InteractionResponse>, StatusCode> {
    let timestamp = headers
        .get(HEADER_TIMESTAMP)
        .map(axum::http::HeaderValue::as_bytes)
        .ok_or(StatusCode::BAD_REQUEST)?;

    let signature = headers
        .get(HEADER_SIGNATURE)
        .map(axum::http::HeaderValue::as_bytes)
        .and_then(|x| hex::decode(x).ok())
        .and_then(|x| Signature::from_slice(&x).ok())
        .ok_or(StatusCode::BAD_REQUEST)?;

    let msg = [timestamp, &body].concat();

    PUBLIC_KEY
        .verify(msg, &signature)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    interactions_dispatch(&body).await
}

#[allow(clippy::unused_async)]
async fn interactions_dispatch(body: &Bytes) -> Result<Json<InteractionResponse>, StatusCode> {
    let interaction: Interaction =
        serde_json::from_slice(body).map_err(|_| StatusCode::BAD_REQUEST)?;

    match (interaction.kind, interaction.data) {
        (Ping, _) => Ok(Json(InteractionResponse {
            kind: InteractionResponseType::Pong,
            data: None,
        })),

        _ => Err(StatusCode::BAD_REQUEST),
    }
}
