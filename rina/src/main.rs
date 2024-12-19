use clap::{command, Parser};
use rig::providers::{self, openai};
use rina_core::attention::{Attention, AttentionConfig};
use rina_core::knowledge::Document;

use rina_core::character;
use rina_core::init_logging;
use rina_core::knowledge::KnowledgeBase;
use rina_core::loaders::github::GitLoader;
use rina_core::{agent::Agent, clients::discord::DiscordClient, clients::twitter::TwitterClient, clients::telegram::TelegramClient};
use sqlite_vec::sqlite3_vec_init;
use tokio_rusqlite::ffi::sqlite3_auto_extension;
use tokio_rusqlite::Connection;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {

    /// Path to character profile TOML file
    #[arg(long, default_value = "rina/src/characters/rina.toml")]
    character: String,

    /// Path to database
    #[arg(long, default_value = ":memory:")]
    db_path: String,

    /// Discord API token (can also be set via DISCORD_API_TOKEN env var)
    #[arg(long, env = "DISCORD_API_TOKEN", default_value = "")]
    discord_api_token: String,

    /// OpenAI API token (can also be set via OPENAI_API_KEY env var)
    #[arg(long, env = "OPENAI_API_KEY", default_value = "")]
    openai_api_key: String,

    /// GitHub repository URL
    #[arg(long, default_value = "https://github.com/cartridge-gg/docs")]
    github_repo: String,

    /// Local path to clone GitHub repository
    #[arg(long, default_value = ".repo")]
    github_path: String,
    /// Twitter username
    #[arg(long, env = "TWITTER_USERNAME")]
    twitter_username: String,

    /// Twitter password
    #[arg(long, env = "TWITTER_PASSWORD")]
    twitter_password: String,

    /// Twitter email (optional, for 2FA)
    #[arg(long, env = "TWITTER_EMAIL")]
    twitter_email: Option<String>,

    /// Twitter 2FA code (optional)
    #[arg(long, env = "TWITTER_2FA_CODE")]
    twitter_2fa_code: Option<String>,

    /// Twitter cookie string (optional, alternative to username/password)
    #[arg(long, env = "TWITTER_COOKIE_STRING")]
    twitter_cookie_string: Option<String>,

    #[arg(long, env = "HEURIST_API_KEY")]
    heurist_api_key: Option<String>,

    /// Telegram bot token
    #[arg(long, env = "TELEGRAM_BOT_TOKEN")]
    telegram_bot_token: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    dotenv::dotenv().ok();

    let args = Args::parse();

    let repo = GitLoader::new(args.github_repo, &args.github_path)?;

    let character_content =
        std::fs::read_to_string(&args.character).expect("Failed to read character file");
    
    let character: character::Character = toml::from_str(&character_content)
        .map_err(|e| format!("Failed to parse character TOML: {}\nContent: {}", e, character_content))?;

    let oai = providers::openai::Client::new(&args.openai_api_key);
    let embedding_model = oai.embedding_model(openai::TEXT_EMBEDDING_3_LARGE);
    let completion_model = oai.completion_model(openai::GPT_4O);
    let should_respond_completion_model = oai.completion_model(openai::GPT_4O);

    // Initialize the `sqlite-vec`extension
    // See: https://alexgarcia.xyz/sqlite-vec/rust.html
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(sqlite3_vec_init as *const ())));
    }

    let conn = Connection::open(args.db_path).await?;
    let mut knowledge = KnowledgeBase::new(conn.clone(), embedding_model).await?;

    knowledge
        .add_documents(
            repo.with_dir("src/pages/vrf")?
                .read_with_path()
                .ignore_errors()
                .into_iter()
                .map(|(path, content)| Document {
                    id: path.to_string_lossy().to_string(),
                    source_id: "github".to_string(),
                    content,
                    created_at: chrono::Utc::now(),
                }),
        )
        .await?;

    let agent = Agent::new(character, completion_model, knowledge);

    let config = AttentionConfig {
        bot_names: vec![agent.character.name.clone()],
        ..Default::default()
    };
    let attention = Attention::new(config, should_respond_completion_model);
    let telegram = TelegramClient::new(agent.clone(), attention.clone(), args.telegram_bot_token);
    let discord = DiscordClient::new(agent.clone(), attention.clone());
    let twitter = TwitterClient::new(
        agent.clone(),
        attention.clone(),
        args.twitter_username,
        args.twitter_password,
        args.twitter_email,
        args.twitter_2fa_code,
        args.twitter_cookie_string,
        args.heurist_api_key,
    ).await?;

    tokio::join!(
        telegram.start(),
        discord.start(&args.discord_api_token),
        twitter.start()
    );
    Ok(())
}
