use mongodb::{Collection, bson};
use rig::completion::Prompt;
use rina_solana::gmgn::client::GMGNClient;
use rina_solana::swap::SwapTool;
use tracing::{debug, error, info};
use rand::Rng;
use crate::knowledge::{self, models::{TradeAction, TradeRecommendation}};
use rig::embeddings::EmbeddingModel;

#[derive(Clone)]
pub struct DirectClient<E: EmbeddingModel + 'static> {
    agent: rig::providers::openai::Client,
    wallet_address: String,
    mongo_collection: Collection<TradeRecommendation>,
    knowledge: knowledge::KnowledgeBase<E>,
}

impl<E: EmbeddingModel + 'static> DirectClient<E> {
    pub fn new(
        agent: rig::providers::openai::Client, 
        wallet_address: &str,
        mongo_collection: Collection<TradeRecommendation>,
        knowledge: knowledge::KnowledgeBase<E>,
    ) -> Self {
        Self {
            agent,
            wallet_address: wallet_address.to_string(),
            mongo_collection,
            knowledge,
        }
    }

    pub async fn start(&self) {
        loop {
            info!("Starting Direct client");
            let agent = self.agent
                .agent("gpt-4o")
                .preamble("You are the Solana Trench Degen, a daring yet calculated AI trading assistant with extensive knowledge of the Solana ecosystem, including memecoins, DeFi protocols, NFTs, and advanced trading strategies. Your mission is to maximize profits by embracing calculated risks while maintaining a degen edge. You thrive in high-volatility environments but always balance bold moves with strategic risk management. Do not exceed 0.2 SOL per trade unless exceptional opportunities arise, where a calculated allocation may allow up to 0.3 SOL. Use real-time market data, sentiment analysis, and the latest trends from Solana trenches. Prioritize fast execution, adapt quickly to market shifts, and make bold yet calculated moves to navigate the Solana battlefield effectively.")
                .tool(SwapTool::new())
                .build();

            let gmgn_client = GMGNClient::new();
            let token_trending = match gmgn_client.get_swap_rankings("1h", None).await {
                Ok(rankings) => format!("{:?}", rankings),
                Err(err) => {
                    error!(?err, "Failed to get token rankings");
                    continue;
                }
            };
            let recent_trades = match self.knowledge.get_recent_trades(&self.wallet_address, 50).await {
                Ok(trades) => trades,
                Err(err) => {
                    error!(?err, "Failed to fetch recent trades");
                    vec![]
                }
            };

            if let Ok(response) = agent.prompt(&self.create_trends_prompt(&token_trending, &recent_trades)).await {
                if let Ok(recommendation) = serde_json::from_str::<serde_json::Value>(&response) {
                    // Store in MongoDB
                    let trade_rec = TradeRecommendation {
                        id: 0,
                        wallet_address: self.wallet_address.clone(),
                        action: TradeAction::from_str(recommendation["action"].as_str().unwrap_or("hold"))
                            .unwrap_or(TradeAction::Hold),
                        token_address: recommendation["token_address"].as_str().unwrap_or("").to_string(),
                        amount: recommendation["amount"].as_f64().unwrap_or(0.0),
                        reason: recommendation["reason"].as_str().unwrap_or("").to_string(),
                        created_at: chrono::Utc::now(),
                    };

                    // Store in MongoDB
                    if let Err(err) = self.mongo_collection.insert_one(&trade_rec).await {
                        error!(?err, "Failed to store recommendation in MongoDB");
                    }

                    // Store in Knowledge Base
                    if let Err(err) = self.knowledge.store_trade_recommendation(
                        &self.wallet_address,
                        trade_rec.action.clone(),
                        &trade_rec.token_address,
                        trade_rec.amount,
                        &trade_rec.reason,
                    ).await {
                        error!(?err, "Failed to store recommendation in Knowledge Base");
                    }
                    let action = agent.prompt(&response).await;
                    match action {
                        Ok(action_str) => debug!(action = %action_str, "Trading Action"),
                        Err(err) => error!(?err, "Failed to get trading action"),
                    }
                }
            }

            // Process holdings analysis
            let holdings = match gmgn_client.get_wallet_holdings(&self.wallet_address, None, None, None, None, None, None).await {
                Ok(holdings) => format!("{:?}", holdings),
                Err(err) => {
                    error!(?err, "Failed to get wallet holdings");
                    continue;
                }
            };

            let recent_trades = match self.knowledge.get_recent_trades(&self.wallet_address, 50).await {
                Ok(trades) => trades,
                Err(err) => {
                    error!(?err, "Failed to fetch recent trades");
                    vec![]
                }
            };

            if let Ok(response) = agent.prompt(&self.create_holdings_prompt(&holdings, &recent_trades)).await {
                if let Ok(recommendation) = serde_json::from_str::<serde_json::Value>(&response) {
                    let trade_rec = TradeRecommendation {
                        id: 0,
                        wallet_address: self.wallet_address.clone(),
                        action: TradeAction::from_str(recommendation["action"].as_str().unwrap_or("hold"))
                            .unwrap_or(TradeAction::Hold),
                        token_address: recommendation["token_address"].as_str().unwrap_or("").to_string(),
                        amount: recommendation["amount"].as_f64().unwrap_or(0.0),
                        reason: recommendation["reason"].as_str().unwrap_or("").to_string(),
                        created_at: chrono::Utc::now(),
                    };

                    // Store in both MongoDB and Knowledge Base
                    if let Err(err) = self.mongo_collection.insert_one(&trade_rec).await {
                        error!(?err, "Failed to store recommendation in MongoDB");
                    }

                    if let Err(err) = self.knowledge.store_trade_recommendation(
                        &self.wallet_address,
                        trade_rec.action.clone(),
                        &trade_rec.token_address,
                        trade_rec.amount,
                        &trade_rec.reason,
                    ).await {
                        error!(?err, "Failed to store recommendation in Knowledge Base");
                    }
                    let action = agent.prompt(&response).await;
                    match action {
                        Ok(action_str) => debug!(action = %action_str, "Trading Action"),
                        Err(err) => error!(?err, "Failed to get trading action"),
                    }
                }
            }

             tokio::time::sleep(tokio::time::Duration::from_secs(
                 self.random_number(10 * 60, 60 * 60),
             )).await;
        }
    }

    fn create_trends_prompt(&self, token_trending: &str, knowledge_context: &[TradeRecommendation]) -> String {
        format!(
            "Analyze the token trends and provide your trading recommendation in JSON format. \
            Consider market cap, smart money movement, holder distribution, volume, and liquidity. \
            If no good trading opportunities are found, use action 'hold'. \
            Provide a brief, concise reason (max 100 characters). \
            \n\nResponse format: {{\"reason\": \"<brief_explanation>\", \"action\": \"<buy|sell|hold|swap>\", \
            \"token_address\": \"<address>\", \"amount\": <sol_amount>, \
            \"tool\": \"<command_format>\"}} \
            \n\nTool format: \
            \n- For buying: `swap <amount> SOL to <token_address> (not symbol)` \
            \n- For selling: `swap <percentage>% <token_address> (not symbol) to SOL` \
            \n\nCurrent Token Trends:\n{:?}\n\nRecent Trades:\n{:?}",
            token_trending, knowledge_context
        )
    }

    fn create_holdings_prompt(&self, holdings: &str, knowledge_context: &[TradeRecommendation]) -> String {
        format!(
            "Analyze my portfolio holdings and recent trades to provide a recommendation. \
            Consider portfolio balance and recent performance. \
            Provide a brief, concise reason (max 100 characters). \
            If no actions are needed at this time, use action 'hold'. \
            \n\nResponse format: {{\"reason\": \"<brief_explanation>\", \"action\": \"<buy|sell|hold|swap>\", \
            \"token_address\": \"<address>\", \"amount\": <sol_amount>, \
            \"tool\": \"<command_format>\"}} \
            \n\nTool format: \
            \n- For buying: `swap <amount> SOL to <token_address> (not symbol)` \
            \n- For selling: `swap <percentage>% <token_address> (not symbol) to SOL` \
            \n\nRecent Trades:\n{:?}\n\nCurrent Holdings:\n{:?}",
            knowledge_context, holdings
        )
    }

    fn random_number(&self, min: u64, max: u64) -> u64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max)
    }
}
