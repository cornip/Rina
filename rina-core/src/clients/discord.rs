use rig::{
    completion::{CompletionModel, Prompt},
    embeddings::EmbeddingModel,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::GatewayIntents;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use std::collections::HashSet;
use tracing::{debug, error, info};

use crate::{agent::Agent, attention::AttentionCommand};
use crate::{
    attention::{Attention, AttentionContext},
    knowledge,
};

const MIN_CHUNK_LENGTH: usize = 100;
const MAX_MESSAGE_LENGTH: usize = 1500;
const MAX_HISTORY_MESSAGES: i64 = 10;

#[derive(Clone)]
pub struct DiscordClient<M: CompletionModel, E: EmbeddingModel + 'static> {
    agent: Agent<M, E>,
    attention: Attention<M>,
}

impl<M: CompletionModel + 'static, E: EmbeddingModel + 'static> DiscordClient<M, E> {
    pub fn new(agent: Agent<M, E>, attention: Attention<M>) -> Self {
        Self { agent, attention }
    }

    pub async fn start(&self, token: &str) {
        info!("Starting Discord bot");
        
        let intents = GatewayIntents::GUILD_MESSAGES
            | GatewayIntents::DIRECT_MESSAGES
            | GatewayIntents::MESSAGE_CONTENT;

        let mut client = Client::builder(token, intents)
            .event_handler(self.clone())
            .await
            .expect("Error creating Discord client");

        if let Err(why) = client.start().await {
            error!(?why, "Discord client error");
        }
    }
}

impl From<Message> for knowledge::Message {
    fn from(msg: Message) -> Self {
        Self {
            id: msg.id.to_string(),
            source: knowledge::Source::Discord,
            source_id: msg.author.id.to_string(),
            channel_type: if msg.guild_id.is_none() {
                knowledge::ChannelType::DirectMessage
            } else {
                knowledge::ChannelType::Text
            },
            channel_id: msg.channel_id.to_string(),
            account_id: msg.author.id.to_string(),
            role: "user".to_string(),
            content: msg.content.clone(),
            created_at: *msg.timestamp,
        }
    }
}

#[async_trait]
impl<M: CompletionModel + 'static, E: EmbeddingModel + 'static> EventHandler
    for DiscordClient<M, E>
{
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.author.bot {
            return;
        }

        let knowledge = self.agent.knowledge();
        let knowledge_msg = knowledge::Message::from(msg.clone());

        if let Err(err) = knowledge
            .clone()
            .create_message(knowledge_msg.clone())
            .await
        {
            error!(?err, "Failed to store message");
            return;
        }

        debug!("Fetching message history for channel {}", msg.channel_id);
        let history = match knowledge
            .channel_messages(&msg.channel_id.to_string(), MAX_HISTORY_MESSAGES)
            .await
        {
            Ok(messages) => {
                debug!(message_count = messages.len(), "Retrieved message history");
                messages
            }
            Err(err) => {
                error!(?err, "Failed to fetch recent messages");
                return;
            }
        };

        let mentioned_names: HashSet<String> =
            msg.mentions.iter().map(|user| user.name.clone()).collect();
        debug!(
            mentioned_names = ?mentioned_names,
            "Mentioned names in message"
        );

        let context = AttentionContext {
            message_content: msg.content.clone(),
            mentioned_names,
            history,
            channel_type: knowledge_msg.channel_type,
            source: knowledge_msg.source,
        };

        debug!(?context, "Attention context");

        match self.attention.should_reply(&context).await {
            AttentionCommand::Respond => {}
            _ => {
                debug!("Bot decided not to reply to message");
                return;
            }
        }

        let agent = self
            .agent
            .builder()
            .context(&format!(
                "Current time: {}",
                chrono::Local::now().format("%I:%M:%S %p, %Y-%m-%d")
            ))
            .context("Please keep your responses concise and under 2000 characters when possible.")
            .build();

        let response = match agent.prompt(&msg.content).await {
            Ok(response) => response,
            Err(err) => {
                error!(?err, "Failed to generate response");
                return;
            }
        };

        debug!(response = %response, "Generated response");

        let chunks = chunk_message(&response, MAX_MESSAGE_LENGTH, MIN_CHUNK_LENGTH);

        for chunk in chunks {
            if let Err(why) = msg.channel_id.say(&ctx.http, chunk).await {
                error!(?why, "Failed to send message");
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!(name = self.agent.character.name, "Bot connected");
        info!(guild_count = ready.guilds.len(), "Serving guilds");
    }
}

pub fn chunk_message(text: &str, max_length: usize, min_chunk_length: usize) -> Vec<String> {
    // Base case: if text is shorter than min_chunk_length, return as single chunk
    if text.len() <= min_chunk_length {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();

    // Find split point for current chunk
    let mut split_index = text.len();
    let mut in_heading = false;

    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Start new chunk on headings
        if line.starts_with('#') {
            if i > 0 {
                split_index = text.find(line).unwrap_or(text.len());
                in_heading = true;
                break;
            }
        }

        // Check if adding this line would exceed max_length
        let line_start = text.find(line).unwrap_or(text.len());
        if line_start + line.len() > max_length && i > 0 {
            split_index = line_start;
            break;
        }
    }

    // Split text and recurse
    if split_index < text.len() {
        let (chunk, rest) = text.split_at(split_index);
        let mut chunk = chunk.trim().to_string();

        // Add newline after chunk if we're not splitting on a heading
        if !in_heading && !rest.trim().starts_with('#') {
            chunk.push('\n');
        }

        // Strip trailing newline if it's the last character
        if chunk.ends_with('\n') {
            chunk.pop();
        }

        chunks.push(chunk);
        chunks.extend(chunk_message(rest.trim(), max_length, min_chunk_length));
    } else {
        chunks.push(text.trim().to_string());
    }

    chunks
}


