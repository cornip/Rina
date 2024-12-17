use crate::{
    agent::Agent,
    attention::{Attention, AttentionCommand, AttentionContext},
    knowledge::{self, ChannelType, Source},
};
use rig::{completion::{CompletionModel, Prompt}, embeddings::EmbeddingModel};
use std::collections::HashSet;
use teloxide::{
    prelude::*,
    types::MessageKind,
};
use tracing::{debug, error, info};

const MAX_HISTORY_MESSAGES: i64 = 10;

pub struct TelegramClient<M: CompletionModel, E: EmbeddingModel + 'static> {
    agent: Agent<M, E>,
    attention: Attention<M>,
    bot: Bot,
}

impl<M: CompletionModel + 'static, E: EmbeddingModel + 'static> TelegramClient<M, E> {
    pub fn new(agent: Agent<M, E>, attention: Attention<M>, token: String) -> Self {
        let bot = Bot::new(token);
        Self {
            agent,
            attention,
            bot,
        }
    }

    pub async fn start(&self) {
        info!("Starting Telegram bot");
        let this = self.clone();
        let handler = Update::filter_message().branch(
            dptree::filter(|msg: teloxide::types::Message| matches!(msg.kind, MessageKind::Common(_)))
                .endpoint(move |msg: teloxide::types::Message| {
                    let this = this.clone();
                    async move { this.handle_message(msg).await }
                }),
        );

        Dispatcher::builder(self.bot.clone(), handler)
            .enable_ctrlc_handler()
            .build()
            .dispatch()
            .await;
    }

    async fn handle_message(&self, msg: teloxide::types::Message) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if msg.from().map_or(true, |user| user.is_bot) {
            return Ok(());
        }

        let text = if let Some(text) = msg.text() {
            text.to_string()
        } else {
            return Ok(());
        };

        let knowledge = self.agent.knowledge();
        let knowledge_msg = self.convert_to_knowledge_message(msg.clone());

        if let Err(err) = knowledge.clone().create_message(knowledge_msg.clone()).await {
            error!(?err, "Failed to store message");
            return Ok(());
        }

        debug!("Fetching message history for chat {}", msg.chat.id);
        let history = match knowledge
            .channel_messages(&msg.chat.id.to_string(), MAX_HISTORY_MESSAGES)
            .await
        {
            Ok(messages) => {
                debug!(message_count = messages.len(), "Retrieved message history");
                messages
            }
            Err(err) => {
                error!(?err, "Failed to fetch recent messages");
                return Ok(());
            }
        };

        let mentioned_names = extract_mentions(&text);
        debug!(mentioned_names = ?mentioned_names, "Mentioned names in message");

        let context = AttentionContext {
            message_content: text.clone(),
            mentioned_names,
            history,
            channel_type: if msg.chat.is_private() {
                ChannelType::DirectMessage
            } else {
                ChannelType::Text
            },
            source: Source::Telegram,
        };

        debug!(?context, "Attention context");

        match self.attention.should_reply(&context).await {
            AttentionCommand::Respond => {}
            _ => {
                debug!("Bot decided not to reply to message");
                return Ok(());
            }
        }

        let agent = self
            .agent
            .builder()
            .context(&format!(
                "Current time: {}",
                chrono::Local::now().format("%I:%M:%S %p, %Y-%m-%d")
            ))
            .context("Please keep your responses concise and under 4096 characters when possible.")
            .build();
        let response = match agent.prompt(&text).await {
            Ok(response) => response,
            Err(err) => {
                error!(?err, "Failed to generate response");
                return Ok(());
            }
        };

        debug!(response = %response, "Generated response");

        if let Err(why) = self.bot.send_message(msg.chat.id, response).send().await {
            error!(?why, "Failed to send message");
        }

        Ok(())
    }

    fn convert_to_knowledge_message(&self, msg: teloxide::types::Message) -> knowledge::Message {
        knowledge::Message {
            id: msg.id.to_string(),
            source: Source::Telegram,
            source_id: msg.from().map_or_else(String::new, |user| user.id.to_string()),
            channel_type: if msg.chat.is_private() {
                ChannelType::DirectMessage
            } else {
                ChannelType::Text
            },
            channel_id: msg.chat.id.to_string(),
            account_id: msg.from().map_or_else(String::new, |user| user.id.to_string()),
            role: "user".to_string(),
            content: msg.text().unwrap_or_default().to_string(),
            created_at: msg.date.into(),
        }
    }
}

fn extract_mentions(text: &str) -> HashSet<String> {
    text.split_whitespace()
        .filter(|word| word.starts_with('@'))
        .map(|mention| mention[1..].to_string())
        .collect()
}

impl<M: CompletionModel, E: EmbeddingModel> Clone for TelegramClient<M, E> {
    fn clone(&self) -> Self {
        Self {
            agent: self.agent.clone(),
            attention: self.attention.clone(),
            bot: self.bot.clone(),
        }
    }
}
