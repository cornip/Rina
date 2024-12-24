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
    
    let home_timeline = scraper.get_home_timeline(20, vec![]).await?;
    println!("Home timeline: {:?}", home_timeline);
    Ok(())
}