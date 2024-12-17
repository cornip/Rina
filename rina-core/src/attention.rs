use rig::completion::{CompletionModel, ModelChoice};
use tracing::debug;

use crate::knowledge::{ChannelType, Source};
use std::collections::HashSet;

const RESPOND_COMMAND: &str = "[RESPOND]";
const IGNORE_COMMAND: &str = "[IGNORE]";
const STOP_COMMAND: &str = "[STOP]";

#[derive(Debug, PartialEq)]
pub enum AttentionCommand {
    Respond,
    Ignore,
    Stop,
}

#[derive(Debug)]
pub struct AttentionContext {
    pub message_content: String,
    pub mentioned_names: HashSet<String>,
    pub history: Vec<(String, String)>,
    pub channel_type: ChannelType,
    pub source: Source,
}

#[derive(Clone, Debug)]
pub struct AttentionConfig {
    pub bot_names: Vec<String>,
    pub reply_threshold: f32,
    pub max_history_messages: i64,
    pub cooldown_messages: i64,
}

impl Default for AttentionConfig {
    fn default() -> Self {
        Self {
            bot_names: vec!["shinobai".to_string(), "shinobi".to_string()],
            reply_threshold: 0.6,
            max_history_messages: 10,
            cooldown_messages: 3,
        }
    }
}

#[derive(Clone)]
pub struct Attention<M: CompletionModel> {
    config: AttentionConfig,
    completion_model: M,
}

impl<M: CompletionModel> Attention<M> {
    pub fn new(config: AttentionConfig, completion_model: M) -> Self {
        Self {
            config,
            completion_model,
        }
    }

    pub async fn should_reply(&self, context: &AttentionContext) -> AttentionCommand {
        let content = context.message_content.to_lowercase();

        // Always reply to DMs
        if context.channel_type == ChannelType::DirectMessage {
            return AttentionCommand::Respond;
        }

        // Check for mentions or name references
        for name in &self.config.bot_names {
            let mentioned = context.mentioned_names.contains(name);
            let name_in_content = content.contains(&name.to_lowercase());

            debug!(
                name = name,
                mentioned = mentioned,
                name_in_content = name_in_content,
                "Checking if bot name was mentioned"
            );

            if mentioned || name_in_content {
                debug!("Bot name {} was mentioned, will reply", name);
                return AttentionCommand::Respond;
            }
        }

        // Check for stop/disengage phrases
        let stop_phrases = [
            "shut up",
            "stop",
            "please shut up",
            "shut up please",
            "dont talk",
            "silence",
            "stop talking",
            "be quiet",
            "hush",
            "wtf",
            "stfu",
            "stupid bot",
            "dumb bot",
            "stop responding",
            "can you not",
            "can you stop",
            "be quiet",
        ];

        if stop_phrases.iter().any(|phrase| content.contains(phrase)) {
            return AttentionCommand::Stop;
        }

        // Ignore very short messages
        if content.len() < 4 {
            return AttentionCommand::Ignore;
        }

        // Use LLM to decide if we should respond
        let prompt = format!(
            "You are in a room with other users. You should only respond when addressed or when the conversation is relevant to you.\n\n\
            Response options:\n\
            {RESPOND_COMMAND} - Message is directed at you or conversation is relevant\n\
            {IGNORE_COMMAND} - Message is not interesting or not directed at you\n\
            {STOP_COMMAND} - User wants you to stop or conversation has concluded\n\n\
            Recent messages:\n{}\n\nLatest message: {}\n\n\
            Choose one response option:",
            context.history.iter()
                .map(|(_, msg)| format!("- {}", msg))
                .collect::<Vec<_>>()
                .join("\n"),
            context.message_content
        );

        let builder = self.completion_model.completion_request(&prompt);

        match self.completion_model.completion(builder.build()).await {
            Ok(response) => match response.choice {
                ModelChoice::Message(text) => {
                    if text.contains(RESPOND_COMMAND) {
                        AttentionCommand::Respond
                    } else if text.contains(STOP_COMMAND) {
                        AttentionCommand::Stop
                    } else {
                        AttentionCommand::Ignore
                    }
                }
                ModelChoice::ToolCall(_, _) => AttentionCommand::Ignore,
            },
            Err(_) => AttentionCommand::Ignore,
        }
    }
}
