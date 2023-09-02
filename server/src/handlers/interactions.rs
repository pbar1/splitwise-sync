use anyhow::Context;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::Json;
use ed25519_compact::Signature;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::InteractionData as InData;
use twilight_model::application::interaction::InteractionType as InType;
use twilight_model::http::interaction::InteractionResponse;
use twilight_model::http::interaction::InteractionResponseType;

use crate::cmd::server::ServerState;

const HEADER_SIGNATURE: &str = "X-Signature-Ed25519";
const HEADER_TIMESTAMP: &str = "X-Signature-Timestamp";

pub async fn interactions(
    state: State<ServerState>,
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

    state
        .public_key
        .verify(msg, &signature)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    interactions_dispatch(state, &body).await
}

async fn interactions_dispatch(
    state: State<ServerState>,
    body: &Bytes,
) -> Result<Json<InteractionResponse>, StatusCode> {
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

            // FIXME: Pretend we already synced to Splitwise and delete the message
            let channel_id = interaction
                .channel
                .context("channel was empty")
                .map_err(|_| StatusCode::BAD_REQUEST)?
                .id;
            let message_id = interaction
                .message
                .context("message was empty")
                .map_err(|_| StatusCode::BAD_REQUEST)?
                .id;

            let client = twilight_http::Client::new(state.bot_token.clone());

            tracing::info!("deleting processed message");
            client
                .delete_message(channel_id, message_id)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            tracing::info!(%message_id, %channel_id, "message was deleted");

            Ok(Json(InteractionResponse {
                kind: InteractionResponseType::DeferredUpdateMessage,
                data: None,
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
