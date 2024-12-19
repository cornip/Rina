use crate::{
    agent::Agent,
    attention::{Attention, AttentionCommand, AttentionContext},
    knowledge::{ChannelType, Message, Source},
};

use rand::Rng;
use rig::{
    completion::{CompletionModel, Prompt},
    embeddings::EmbeddingModel,
};
use rig_twitter::scraper::Scraper;
use std::collections::HashSet;
use std::sync::Arc;
use tracing::{debug, error, info};
use crate::clients::heuris::HeurisClient;
const MAX_TWEET_LENGTH: usize = 280;
const MAX_HISTORY_TWEETS: i64 = 10;

pub struct TwitterClient<M: CompletionModel, E: EmbeddingModel + 'static> {
    agent: Agent<M, E>,
    attention: Attention<M>,
    scraper: Scraper,
    username: String,
    heurist_api_key: Option<String>,
}

impl From<rig_twitter::models::Tweet> for Message {
    fn from(tweet: rig_twitter::models::Tweet) -> Self {
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

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        info!("Starting Twitter bot");
        self.start_twitter().await?;
        Ok(())
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
    async fn start_twitter(&self) -> Result<(), Box<dyn std::error::Error>> {
        loop {
             self.post_new_tweet().await?;

            let mentions = self
                .scraper
                .search_tweets(
                    &format!("@{}", self.username),
                    10,
                    rig_twitter::search::SearchMode::Latest,
                    None,
                )
                .await?;
            for tweet in mentions.tweets {
                self.handle_mention(tweet).await?;
                // Random delay between 30 and 60 seconds
                tokio::time::sleep(tokio::time::Duration::from_secs(self.random_number(30, 60)))
                    .await;
            }
            // Random delay between 30 minutes and 1 hour
            tokio::time::sleep(tokio::time::Duration::from_secs(
                self.random_number(30 * 60, 60 * 60),
            ))
            .await;
        }
    }

    async fn handle_mention(
        &self,
        tweet: rig_twitter::models::Tweet,
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

        let agent = self
            .agent
            .builder()
            .context(&format!(
                "Current time: {}",
                chrono::Local::now().format("%I:%M:%S %p, %Y-%m-%d")
            ))
            .context("Please keep your responses concise and under 280 characters.")
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
        tweet: &rig_twitter::models::Tweet,
    ) -> Result<Vec<rig_twitter::models::Tweet>, Box<dyn std::error::Error>> {
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
}
