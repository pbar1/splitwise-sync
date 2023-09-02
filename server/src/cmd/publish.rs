use clap::Args;
use twilight_model::channel::message::component::ActionRow;
use twilight_model::channel::message::component::Button;
use twilight_model::channel::message::component::ButtonStyle;
use twilight_model::channel::message::component::Component;
use twilight_model::id::marker::ChannelMarker;
use twilight_model::id::Id;

#[derive(Debug, Args)]
pub struct PublishArgs {
    /// Transaction ID
    #[arg(long, short = 'i')]
    id: String,

    /// Transaction date
    #[arg(long, short = 'd')]
    date: String,

    /// Transaction description
    #[arg(long, short = 's')]
    description: String,

    /// Transaction amount
    #[arg(long, short = 'a')]
    amount: String,

    /// Token to authenticate with Discord
    #[arg(long, env = "DISCORD_BOT_TOKEN")]
    bot_token: String,

    /// ID of the Discord channel to publish messages to
    #[arg(long, env = "DISCORD_CHANNEL_ID")]
    channel_id: Id<ChannelMarker>,
}

impl PublishArgs {
    pub async fn run(&self) -> anyhow::Result<()> {
        let client = twilight_http::Client::new(self.bot_token.clone());

        let button = Component::ActionRow(ActionRow {
            components: Vec::from([
                Component::Button(Button {
                    custom_id: Some(format!("accept:{}", self.id)),
                    disabled: false,
                    emoji: None,
                    label: Some("Accept".to_owned()),
                    style: ButtonStyle::Primary,
                    url: None,
                }),
                Component::Button(Button {
                    custom_id: Some(format!("ignore:{}", self.id)),
                    disabled: false,
                    emoji: None,
                    label: Some("Ignore".to_owned()),
                    style: ButtonStyle::Secondary,
                    url: None,
                }),
            ]),
        });

        let content: String = vec![
            "New transaction! Sync to Splitwise?",
            &format!("- Date: {}", self.date),
            &format!("- Amount: {}", self.amount),
            &format!("- Description: {}", self.description),
        ]
        .join("\n");

        let response = client
            .create_message(self.channel_id)
            .content(&content)?
            .components(&[button])?
            .tts(true) // FIXME: What is this?
            .await?;

        tracing::debug!(?response, "received create message response");

        Ok(())
    }
}
