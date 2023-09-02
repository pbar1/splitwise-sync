use axum::body::Bytes;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::Json;
use ed25519_compact::PublicKey;
use ed25519_compact::Signature;
use once_cell::sync::Lazy;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::InteractionData as InData;
use twilight_model::application::interaction::InteractionType as InType;
use twilight_model::http::interaction::InteractionResponse;
use twilight_model::http::interaction::InteractionResponseType;
use twilight_util::builder::InteractionResponseDataBuilder;

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
        (InType::Ping, _) => {
            tracing::debug!("received Ping interaction");
            Ok(Json(InteractionResponse {
                kind: InteractionResponseType::Pong,
                data: None,
            }))
        }

        (InType::ApplicationCommand, Some(InData::ApplicationCommand(data))) => {
            tracing::debug!(?data, "received ApplicationCommand interaction");
            Err(StatusCode::NOT_IMPLEMENTED)
        }

        (InType::MessageComponent, Some(InData::MessageComponent(data))) => {
            tracing::debug!(?data, "received MessageComponent interaction");

            // Assume the `custom_id` of the component is a transaction ID
            let transaction_id = data.custom_id;
            tracing::info!(%transaction_id, "found transaction ready to sync");

            // FIXME: Wonder what this will even do
            let response_data = InteractionResponseDataBuilder::new().build();

            Ok(Json(InteractionResponse {
                kind: InteractionResponseType::DeferredUpdateMessage,
                data: Some(response_data),
            }))
        }

        (InType::ApplicationCommandAutocomplete, Some(InData::ApplicationCommand(data))) => {
            tracing::debug!(?data, "received ApplicationCommandAutocomplete interaction");
            Err(StatusCode::NOT_IMPLEMENTED)
        }

        (InType::ModalSubmit, Some(InData::ModalSubmit(data))) => {
            tracing::debug!(?data, "received ModalSubmit interaction");
            Err(StatusCode::NOT_IMPLEMENTED)
        }

        _ => Err(StatusCode::BAD_REQUEST),
    }
}
