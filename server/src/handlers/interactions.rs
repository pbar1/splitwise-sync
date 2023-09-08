use anyhow::Context;
use axum::body::Bytes;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::StatusCode;
use axum::Json;
use chrono::NaiveDate;
use chrono::NaiveTime;
use chrono::TimeZone;
use chrono::Utc;
use ed25519_compact::Signature;
use once_cell::sync::Lazy;
use regex::Regex;
use splitwise::model::expenses::CreateExpenseRequest;
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use twilight_model::application::interaction::Interaction;
use twilight_model::application::interaction::InteractionData as InData;
use twilight_model::application::interaction::InteractionType as InType;
use twilight_model::channel::Message;
use twilight_model::http::interaction::InteractionResponse;
use twilight_model::http::interaction::InteractionResponseType;

use crate::cmd::server::ServerState;

const HEADER_SIGNATURE: &str = "X-Signature-Ed25519";
const HEADER_TIMESTAMP: &str = "X-Signature-Timestamp";

static REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"Date: (.*?)\n- Amount: (.*?)\n- Description: (.*?)$")
        .expect("unable to compile regex")
});

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

    Box::pin(interactions_dispatch(state, &body)).await
}

async fn interactions_dispatch(
    state: State<ServerState>,
    body: &Bytes,
) -> Result<Json<InteractionResponse>, StatusCode> {
    let interaction: Interaction =
        serde_json::from_slice(body).map_err(|_| StatusCode::BAD_REQUEST)?;

    match (interaction.kind, interaction.clone().data) {
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

            handle_message_component(state, interaction, data)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?; // FIXME: Map client vs server errors

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

async fn handle_message_component(
    state: State<ServerState>,
    interaction: Interaction,
    data: MessageComponentInteractionData,
) -> anyhow::Result<()> {
    // Assume the `custom_id` of the component is of the form
    // "<accept|ignore>:<transaction id>"
    let mut custom_id = data.custom_id.split(':');
    let action = custom_id.next().context("no colon found in custom_id")?;
    let transaction_id = custom_id
        .next()
        .context("nothing after colon in custom_id")?;

    tracing::info!(%transaction_id, "found transaction ready to sync");

    let message = interaction.message.context("message was empty")?;

    if action == "accept" {
        create_splitwise_expense(state.clone(), transaction_id, &message).await?;
    }

    let channel_id = interaction.channel.context("channel was empty")?.id;
    let message_id = message.id;

    // FIXME: Instead of deleting the message entirely, keeping it while removing
    // the buttons and adding a note of whether it was accepted or ignored would be
    // nice
    tracing::info!("deleting processed message");
    let client = twilight_http::Client::new(state.bot_token.clone());
    client.delete_message(channel_id, message_id).await?;
    tracing::info!(%message_id, %channel_id, "message was deleted");

    Ok(())
}

async fn create_splitwise_expense(
    state: State<ServerState>,
    transaction_id: &str,
    message: &Message,
) -> anyhow::Result<()> {
    let group_id = state.splitwise_group_id;

    let captures = REGEX
        .captures(&message.content)
        .context("unable to match regex")?;

    let date = captures
        .get(1)
        .context("date not captured")
        .and_then(|x| Ok(NaiveDate::parse_from_str(x.as_str(), "%Y-%m-%d")?))?;
    let amount = captures
        .get(2)
        .context("amount not captured")?
        .as_str()
        .replace('-', ""); // Can't be negative
    let description = captures
        .get(3)
        .context("description not captured")?
        .as_str();

    let splitwise_client = splitwise::client::Client::default();

    tracing::info!(
        ?date,
        ?amount,
        ?description,
        ?group_id,
        ?transaction_id,
        "creating splitwise expense"
    );
    let expenses = splitwise_client
        .expenses()
        .create_expense(CreateExpenseRequest {
            cost: amount,
            description: description.to_owned(),
            details: Some(format!("mint:{}", transaction_id)),
            date: Utc.from_utc_datetime(&date.and_time(NaiveTime::default())), // FIXME: UTC offset is weird
            repeat_interval: "never".to_string(),
            currency_code: "USD".to_string(),
            category_id: 0,
            group_id,
            split_equally: true,
            users: None,
        })
        .await?;
    tracing::debug!(?expenses, ?transaction_id, "created splitwise expenses");

    Ok(())
}
