#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub enum Source {
    Discord,
    Telegram,
    Github,
    X,
    Twitter,
}

impl Source {
    pub fn as_str(&self) -> &'static str {
        match self {
            Source::Discord => "discord",
            Source::Telegram => "telegram",
            Source::Github => "github",
            Source::X => "x",
            Source::Twitter => "twitter",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "discord" => Some(Source::Discord),
            "telegram" => Some(Source::Telegram),
            "github" => Some(Source::Github),
            "x" => Some(Source::X),
            "twitter" => Some(Source::Twitter),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub enum ChannelType {
    DirectMessage,
    Text,
    Voice,
    Thread,
}

impl ChannelType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChannelType::DirectMessage => "direct_message",
            ChannelType::Text => "text",
            ChannelType::Voice => "voice",
            ChannelType::Thread => "thread",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "direct_message" => Some(ChannelType::DirectMessage),
            "text" => Some(ChannelType::Text),
            "voice" => Some(ChannelType::Voice),
            "thread" => Some(ChannelType::Thread),
            _ => None,
        }
    }
}

pub trait MessageMetadata {
    fn id(&self) -> String;
    fn source_id(&self) -> String;
    fn channel_id(&self) -> String;
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
    fn source(&self) -> Source;
    fn channel_type(&self) -> ChannelType;
}

pub trait MessageContent {
    fn content(&self) -> &str;
} 