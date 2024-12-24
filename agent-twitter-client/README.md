# Twitter Scraper Library

A Rust port of the TypeScript library (https://github.com/ai16z/agent-twitter-client). This package does not require the Twitter API to use!

A Rust library that provides a scraper interface for the Twitter API. Easily interact with Twitter through authentication, timeline fetching, user operations, and more.

## Features

- Authentication with cookies
- Comprehensive user profile management
- Timeline retrieval
- Tweet interactions (like, retweet, post)
- Advanced search capabilities
- User relationship management (follow/unfollow)

## Installation

Add these dependencies to your `Cargo.toml`:

```toml
[dependencies]
agent-twitter-client = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
dotenv = "0.15"
```

## Quick Start

### Authentication

#### Method 1: Login with Credentials

```rust
use agent_twitter_client::scraper::Scraper;
use dotenv::dotenv;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let mut scraper = Scraper::new().await?;
    
    scraper.login(
        env::var("TWITTER_USERNAME")?,
        env::var("TWITTER_PASSWORD")?,
        Some(env::var("TWITTER_EMAIL")?),
        Some(env::var("TWITTER_2FA_SECRET")?)
    ).await?;
    
    Ok(())
}
```

#### Method 2: Login with Cookie String

```rust
use agent_twitter_client::scraper::Scraper;
use agent_twitter_client::error::Result;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut scraper = Scraper::new().await?;
    let cookie_string = std::env::var("TWITTER_COOKIE_STRING")
        .expect("TWITTER_COOKIE_STRING environment variable not set");
    scraper.set_from_cookie_string(&cookie_string).await?;
    Ok(())
}
```

### User Operations

```rust
use agent_twitter_client::scraper::Scraper;
use agent_twitter_client::error::Result;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut scraper = Scraper::new().await?;
    let cookie_string = std::env::var("TWITTER_COOKIE_STRING")
        .expect("TWITTER_COOKIE_STRING environment variable not set");
    scraper.set_from_cookie_string(&cookie_string).await?;

    // Follow a user
    scraper.follow_user("Rina_RIG").await?;
    
    // Get user profile
    let profile = scraper.get_profile("Rina_RIG").await?;
    
    // Get user's followers
    let (followers, next_cursor) = scraper.get_followers("Rina_RIG", 20, None).await?;
    
    Ok(())
}
```

### Search Operations

```rust
use agent_twitter_client::scraper::Scraper;
use agent_twitter_client::search::SearchMode;
use agent_twitter_client::error::Result;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut scraper = Scraper::new().await?;
    let cookie_string = std::env::var("TWITTER_COOKIE_STRING")
        .expect("TWITTER_COOKIE_STRING environment variable not set");
    scraper.set_from_cookie_string(&cookie_string).await?;
    
    // Search tweets
    let tweets = scraper.search_tweets(
        "@Rina_RIG",
        20,
        SearchMode::Latest,
        None
    ).await?;
    
    // Search user profiles
    let profiles = scraper.search_profiles("rust", 20, None).await?;
    
    Ok(())
}
```

### Timeline Operations

```rust
use agent_twitter_client::scraper::Scraper;
use agent_twitter_client::error::Result;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut scraper = Scraper::new().await?;
    let cookie_string = std::env::var("TWITTER_COOKIE_STRING")
        .expect("TWITTER_COOKIE_STRING environment variable not set");
    scraper.set_from_cookie_string(&cookie_string).await?;
    
    // Get home timeline
    let tweets = scraper.get_home_timeline(20, vec![]).await?;
    
    // Get user's tweets and replies
    let tweets = scraper.fetch_tweets_and_replies("username", 20, None).await?;
    
    Ok(())
}
```

### Tweet Interactions

```rust
use std::fs::File;
use std::io::Read;
use agent_twitter_client::scraper::Scraper;
use agent_twitter_client::error::Result;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let mut scraper = Scraper::new().await?;
    let cookie_string = std::env::var("TWITTER_COOKIE_STRING")
        .expect("TWITTER_COOKIE_STRING environment variable not set");
    scraper.set_from_cookie_string(&cookie_string).await?;
    
    // Like a tweet
    scraper.like_tweet("tweet_id").await?;
    
    // Retweet
    scraper.retweet("tweet_id").await?;
    
    // Post a new tweet
    scraper.send_tweet("Hello, Twitter!", None, None).await?;

    // Send a simple tweet
    let tweet = scraper.send_tweet("Hello world!", None, None).await?;

    // Create media data tuple with image data and MIME type
    let mut file = File::open("image.jpg")?;
    let mut image_data = Vec::new();
    file.read_to_end(&mut image_data)?;
    let media_data = vec![(image_data, "image/jpeg".to_string())];

    // Send the tweet with the image
    let tweet_with_media = scraper.send_tweet(
        "Check out this image!",
        None,
        Some(media_data)
    ).await?;

    Ok(())
}
```

## Configuration

Create a `.env` file with your credentials:

```env
TWITTER_USERNAME=your_username
TWITTER_PASSWORD=your_password
TWITTER_EMAIL=your_email@example.com
TWITTER_2FA_SECRET=your_2fa_secret  # Optional
TWITTER_COOKIE_STRING='your_cookie_string'
```

## License

Created by [Rina](https://x.com/Rina_RIG)

![banner](https://github.com/user-attachments/assets/b2e37bc8-7fe9-4285-a85b-c41dae9d288b)

## Contributing

We welcome contributions! Please feel free to submit a Pull Request.
