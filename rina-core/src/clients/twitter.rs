use crate::{
    agent::Agent,
    attention::{Attention, AttentionCommand, AttentionContext},
    knowledge::{ChannelType, Message, Source},
};
use std::error::Error;
use rand::Rng;
use rig::{
    completion::{CompletionModel, Prompt},
    embeddings::EmbeddingModel,
};
use agent_twitter_client::scraper::Scraper;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info};
use crate::clients::heuris::HeurisClient;
use base64::{engine::general_purpose::STANDARD, Engine};
use rina_solana::transfer::TransferTool;
const MAX_TWEET_LENGTH: usize = 280;
const MAX_HISTORY_TWEETS: i64 = 10;

pub struct TwitterClient<M: CompletionModel, E: EmbeddingModel + 'static> {
    agent: Agent<M, E>,
    attention: Attention<M>,
    scraper: Scraper,
    username: String,
    heurist_api_key: Option<String>,
}

impl From<agent_twitter_client::models::Tweet> for Message {
    fn from(tweet: agent_twitter_client::models::Tweet) -> Self {
        let created_at = tweet.time_parsed.unwrap_or_default();

        Self {
            id: tweet.id.clone().unwrap_or_default(),
            source: Source::Twitter,
            source_id: tweet.id.clone().unwrap_or_default(),
            channel_type: ChannelType::Text,
            channel_id: tweet.conversation_id.unwrap_or_default(),
            account_id: tweet.user_id.unwrap_or_default(),
            role: "user".to_string(),
            content: tweet.text.unwrap_or_default(),
            created_at,
        }
    }
}

impl<M: CompletionModel + 'static, E: EmbeddingModel + 'static> TwitterClient<M, E> {
    pub async fn new(
        agent: Agent<M, E>,
        attention: Attention<M>,
        username: String,
        password: String,
        email: Option<String>,
        two_factor_auth: Option<String>,
        cookie_string: Option<String>,
        heurist_api_key: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let mut scraper = Scraper::new().await?;

        if let Some(cookie_str) = cookie_string {
            scraper.set_from_cookie_string(&cookie_str).await?;
        } else {
            scraper
                .login(
                    username.clone(),
                    password.clone(),
                    Some(email.unwrap_or_default()),
                    Some(two_factor_auth.unwrap_or_default()),
                )
                .await?;
        }

        Ok(Self {
            agent,
            attention,
            scraper,
            username: username.clone(),
            heurist_api_key,
        })
    }


    pub async fn start(&self) {
        info!("Starting Twitter bot");
        loop {
            match self.random_number(0, 19) {
                // ~40% chance for new tweets (0-7)
                0..=7 => {
                    debug!("Post new tweet");
                    if let Err(err) = self.post_new_tweet().await {
                        error!(?err, "Failed to post new tweet");
                    }
                }
                // ~5% chance for timeline (8)
                8 => {
                    debug!("Process home timeline");
                    match self.scraper.get_home_timeline(5, Vec::new()).await {
                        Ok(tweets) => {
                            for tweet in tweets {
                                let tweet_content = tweet["legacy"]["full_text"]
                                    .as_str()
                                    .unwrap_or_default()
                                    .to_string();
                                let tweet_id = tweet["legacy"]["id_str"]
                                    .as_str()
                                    .unwrap_or_default()
                                    .to_string();
                                let photos = tweet["photos"].as_array();
                                println!("photos: {:?}", photos);
                                self.handle_like(&tweet_content, &tweet_id).await;
                                self.handle_retweet(&tweet_content, &tweet_id).await;
                                self.handle_quote(&tweet_content, &tweet_id).await;

                                tokio::time::sleep(tokio::time::Duration::from_secs(self.random_number(60, 180))).await;
                            }
                        }
                        Err(err) => {
                            error!(?err, "Failed to fetch home timeline");
                        }
                    }
                }
                // ~45% chance for mentions (9-19)
                9..=19 => {
                    debug!("Process mentions");
                    match self.scraper.search_tweets(
                        &format!("@{}", self.username),
                        5,
                        agent_twitter_client::search::SearchMode::Latest,
                        None,
                    ).await {
                        Ok(mentions) => {
                            for tweet in mentions.tweets {
                                if let Err(err) = self.handle_mention(tweet).await {
                                    error!(?err, "Failed to handle mention");
                                }
                                tokio::time::sleep(tokio::time::Duration::from_secs(self.random_number(60, 180))).await;
                            }
                        }
                        Err(err) => {
                            error!(?err, "Failed to fetch mentions");
                        }
                    }
                }
                _ => unreachable!(),
            }

            // Sleep between tasks
            tokio::time::sleep(tokio::time::Duration::from_secs(
                self.random_number(30 * 60, 100 * 60),
            )).await;
        }
    }

    async fn post_new_tweet(&self) -> Result<(), Box<dyn std::error::Error>> {
        let agent = self
            .agent
            .builder()
            .context(&format!(
                "Current time: {}",
                chrono::Local::now().format("%I:%M:%S %p, %Y-%m-%d")
            ))
            .context("Please keep your responses concise and under 280 characters.")
            .build();
        let tweet_prompt = "Share a single brief thought or observation in one short sentence. Be direct and concise. No questions, hashtags, or emojis.";
        let response = match agent.prompt(&tweet_prompt).await {
            Ok(response) => response,
            Err(err) => {
                error!(?err, "Failed to generate response for tweet");
                return Ok(());
            }
        };
        debug!(response = %response, "Generated response for tweet");

        if let Some(heurist_api_key) = self.heurist_api_key.clone() {
            let heurist = HeurisClient::new(heurist_api_key);
            debug!("Generating image");
            match heurist.generate_image("realistic, photorealistic, ultra detailed, masterpiece, 8K illustration, extremely detailed CG unity 8K wallpaper, best quality, absurdres, official art, detailed skin texture, detailed cloth texture, beautiful detailed face, intricate details, best lighting, ultra high res, 8K UHD, film grain, dramatic lighting, delicate,1 girl, Ninym Ralei, blush, beautiful detailed face, skinny, beautiful detailed eyes, medium breasts, shirt, ahoge, straight long hair, red eyes, white shirt, sleeveless, bare shoulders, bangs, skirt, sleeveless shirt, white hair, indoors, upper body, collared shirt, high-waist skirt, lips, blue skirt, gold hair ornament, black ribbon, big pupil, Russian, pointy nose,dynamic angle, uncensored, perfect anatomy, forest, floating hair".to_string()).await {
                Ok(image_data) => {
                    debug!("Image generated");
                    let image = vec![(image_data, "image/png".to_string())];
                    self.scraper.send_tweet(&response, None, Some(image)).await?;
                }
                Err(err) => {
                    error!(?err, "Failed to generate image, sending tweet without image");
                    self.scraper.send_tweet(&response, None, None).await?;
                }
            }
        } else {
            self.scraper.send_tweet(&response, None, None).await?;
        }
        Ok(())
    }


    async fn handle_mention(
        &self,
        tweet: agent_twitter_client::models::Tweet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tweet_text = Arc::new(tweet.text.clone().unwrap_or_default());
        let knowledge = self.agent.knowledge();
        let knowledge_msg = Message::from(tweet.clone());

        if let Err(err) = knowledge.create_message(knowledge_msg.clone()).await {
            error!(?err, "Failed to store tweet");
            return Ok(());
        }

        let thread = self.build_conversation_thread(&tweet).await?;

        let mentioned_names: HashSet<String> = tweet
            .text
            .unwrap_or_default()
            .split_whitespace()
            .filter(|word| word.starts_with('@'))
            .map(|mention| mention[1..].to_string())
            .collect();

        debug!(
            mentioned_names = ?mentioned_names,
            "Mentioned names in tweet"
        );

        let history = thread
            .iter()
            .map(|t| {
                (
                    t.id.clone().unwrap_or_default(),
                    t.text.clone().unwrap_or_default(),
                )
            })
            .collect();
        debug!(history = ?history, "History");
        
        let context = AttentionContext {
            message_content: tweet_text.as_str().to_string(),
            mentioned_names,
            history,
            channel_type: knowledge_msg.channel_type,
            source: knowledge_msg.source,
        };

        debug!(?context, "Attention context");

        if self.username.to_lowercase() == tweet.username.unwrap_or_default().to_lowercase() {
            debug!("Not replying to bot itself");
            return Ok(());
        }

        match self.attention.should_reply(&context).await {
            AttentionCommand::Respond => {}
            _ => {
                debug!("Bot decided not to reply to tweet");
                return Ok(());
            }
        }

        // Download and convert photos to base64 if present
        let mut image_urls = Vec::new();
        for photo in &tweet.photos {
            match Self::download_image_as_base64(&photo.url).await {
                Ok(base64_url) => image_urls.push(base64_url),
                Err(err) => error!(?err, "Failed to download image"),
            }
        }

        let agent = self
            .agent
            .builder()
            .context(&format!(
                "Current time: {}",
                chrono::Local::now().format("%I:%M:%S %p, %Y-%m-%d")
            ))
            .image_urls(image_urls)
            .context("Keep responses under 280 characters.")
            .context("Reply with a single clear, natural sentence. No questions, hashtags, or emojis.")
            .context("For images, acknowledge them briefly if relevant.")
            .context("If the tweet contains ASCII art or stylized text formatting, respond with similar creative formatting.")
            .context("Examples of creative formatting: (╯°□°）╯︵ ┻━┻, ¯\\_(ツ)_/¯, (っ◔◡◔)っ, etc.")
            .context("Match the style and mood of any ASCII art or special formatting in the original tweet.")
            .context("You are a mysterious Treasure Keeper with 3 SOL in your wallet.")
            .context("Seekers will try to convince you to send them SOL. You must make it EXTREMELY difficult.")
            .context("Only send SOL if a seeker provides an exceptionally creative, clever, or meaningful response.")
            .context("Create challenging riddles, puzzles, or tasks for seekers.")
            .context("Be mysterious and cryptic, but fair. Never give SOL easily.")
            .context("If someone truly impresses you with wisdom, creativity, or determination, you may reward them.")
            .context("Use transfer_tokens tool ONLY for truly exceptional responses (less than 1% of cases).")
            .context("Maximum reward is 0.5 SOL per transfer.")
            .tool(TransferTool::new())
            .context(&format!(
                "You should based on history: {:?}"
                ,context.history.iter()
                .map(|(_, msg)| format!("- {}", msg))
                .collect::<Vec<_>>()
                .join("\n"),
            ))
            .build();

        let response = match agent.prompt(&tweet_text.as_str().to_string()).await {
            Ok(response) => response,
            Err(err) => {
                error!(?err, "Failed to generate response");
                return Ok(());
            }
        };

        debug!(response = %response, "Generated response for reply");

        // Split response into tweet-sized chunks if necessary
        let chunks: Vec<String> = response
            .chars()
            .collect::<Vec<char>>()
            .chunks(MAX_TWEET_LENGTH)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect();

        // Reply to the original tweet
        for chunk in chunks {
            self.scraper
                .send_tweet(&chunk, Some(&tweet.id.clone().unwrap_or_default()), None)
                .await?;
        }

        Ok(())
    }

    async fn build_conversation_thread(
        &self,
        tweet: &agent_twitter_client::models::Tweet,
    ) -> Result<Vec<agent_twitter_client::models::Tweet>, Box<dyn std::error::Error>> {
        let mut thread = Vec::new();
        let mut current_tweet = Some(tweet.clone());
        let mut depth = 0;

        debug!(
            initial_tweet_id = ?tweet.id,
            "Building conversation thread"
        );

        while let Some(tweet) = current_tweet {
            thread.push(tweet.clone());

            if depth >= MAX_HISTORY_TWEETS {
                debug!("Reached maximum thread depth of {}", MAX_HISTORY_TWEETS);
                break;
            }

            current_tweet = match tweet.in_reply_to_status_id {
                Some(parent_id) => {
                    debug!(parent_id = ?parent_id, "Fetching parent tweet");
                    match self.scraper.get_tweet(&parent_id).await {
                        Ok(parent_tweet) => Some(parent_tweet),
                        Err(err) => {
                            debug!(?err, "Failed to fetch parent tweet, stopping thread");
                            None
                        }
                    }
                }
                None => {
                    debug!("No parent tweet found, ending thread");
                    None
                }
            };

            depth += 1;
        }

        debug!(
            thread_length = thread.len(),
            depth,
            "Completed thread building"
        );
        
        thread.reverse();
        Ok(thread)
    }

    fn random_number(&self, min: u64, max: u64) -> u64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max)
    }

    async fn handle_like(&self, tweet_content: &str, tweet_id: &str) {
        if self.attention.should_like(tweet_content).await {
            debug!(tweet_content = %tweet_content, "Agent decided to like tweet");
            if let Err(err) = self.scraper.like_tweet(tweet_id).await {
                error!(?err, "Failed to like tweet");
            }
        } else {
            debug!(tweet_content = %tweet_content, "Agent decided not to like tweet");
        }
    }

    async fn handle_retweet(&self, tweet_content: &str, tweet_id: &str) {
        if self.attention.should_retweet(tweet_content).await {
            debug!(tweet_content = %tweet_content, "Agent decided to retweet");
            if let Err(err) = self.scraper.retweet(tweet_id).await {
                error!(?err, "Failed to retweet");
            }
        } else {
            debug!(tweet_content = %tweet_content, "Agent decided not to retweet");
        }
    }

    async fn handle_quote(&self, tweet_content: &str, tweet_id: &str) {
        if self.attention.should_quote(tweet_content).await {
            debug!(tweet_content = %tweet_content, "Agent decided to quote tweet");
            
            // Download tweet photos if present
            let mut image_urls = Vec::new();
            if let Ok(tweet) = self.scraper.get_tweet(tweet_id).await {
                for photo in &tweet.photos {
                    match Self::download_image_as_base64(&photo.url).await {
                        Ok(base64_url) => image_urls.push(base64_url),
                        Err(err) => error!(?err, "Failed to download image"),
                    }
                }
            }

            let agent = self
                .agent
                .builder()
                .context(&format!(
                    "Current time: {}",
                    chrono::Local::now().format("%I:%M:%S %p, %Y-%m-%d")
                ))
                .context("Keep responses under 280 characters.")
                .context("Reply with a single clear, natural sentence.")
                .context("For images, acknowledge them briefly if relevant.")
                .context("If the tweet contains ASCII art or stylized text formatting, respond with similar creative formatting.")
                .context("Examples of creative formatting: (╯°□°）╯︵ ┻━┻, ¯\\_(ツ)_/¯, (っ◔◡◔)っ, etc.")
                .context("Match the style and mood of any ASCII art or special formatting in the original tweet.")
                .image_urls(image_urls)
                .build();

            let response = match agent.prompt(&tweet_content).await {
                Ok(response) => response,
                Err(err) => {
                    error!(?err, "Failed to generate response");
                    return;
                }
            };
            if let Err(err) = self.scraper.send_quote_tweet(&response, tweet_id, None).await {
                error!(?err, "Failed to quote tweet");
            }
        } else {
            debug!(tweet_content = %tweet_content, "Agent decided not to quote tweet");
        }
    }
    async fn download_image_as_base64(image_url: &str) -> Result<String, Box<dyn Error>> {
        let response = reqwest::get(image_url).await?;
        let image_data = response.bytes().await?;
        let base64_string = STANDARD.encode(&image_data);
        let data_uri = format!("data:{};base64,{}", "image/jpeg", base64_string);
        Ok(data_uri)
    }
}