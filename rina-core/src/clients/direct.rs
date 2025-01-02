use rig::completion::Prompt;
use rina_solana::gmgn::client::GMGNClient;
use rina_solana::swap::SwapTool;
use tracing::{debug, error, info};
use rand::Rng;
#[derive(Clone)]
pub struct DirectClient {
    agent: rig::providers::openai::Client,
    wallet_address: String,
}

impl DirectClient {
    pub fn new(agent: rig::providers::openai::Client, wallet_address: &str) -> Self {
        Self {
            agent,
            wallet_address: wallet_address.to_string(),
        }
    }
    pub async fn start(&self) {
        loop {
            info!("Starting Direct client");
            let agent = self.agent
                .agent("gpt-4o")
            .preamble("You are the Solana Trading memecoin, a sophisticated AI trading assistant with deep knowledge of the Solana ecosystem. You manage a wallet with 1 SOL and must be extremely careful with trades. Do not buy more than 0.3 SOL at a time.")
            .tool(SwapTool::new())
            .build();

            let gmgn_client = GMGNClient::new();
            let token_trending = gmgn_client.get_swap_rankings("1h", None, None, None).await;
            let prompt_analyze_trends = format!(
                "Analyze the token trends and provide your single best trading recommendation. Only give action, no explanation. \
                Consider only tokens that meet ALL these criteria and your risk tolerance: \
                - Market cap between $1M and $5M \
                - Has smart money movement (smart_degen_count > 0) \
                - Has significant bluechip holder presence (bluechip_owner_percentage > 0)
                - Has good holder distribution (holder_count > 100)
                - Has good volume
                - Has good liquidity
                - Consider recent news or events that might impact the token's performance
                - Evaluate the token's historical performance and volatility
                - Assess the token's development activity and community engagement
                \n\nIf no thing good, recommend do nothing. If recommending a trade, use one of these formats, do not use percentage: \
                \n- For buying: `swap <amount> SOL to <token_address> (not symbol)` \
                \n\nCurrent Token Trends:\n{:?}",
                token_trending
            );

            let response_analyze_trends = match agent.prompt(&prompt_analyze_trends).await {
                Ok(response) => response,
                Err(err) => {
                    error!(?err, "Failed to analyze holdings");
                    return;
                }
            };
            let action = agent.prompt(&response_analyze_trends).await;
            match action {
                Ok(action_str) => debug!(action = %action_str, "Trading Action"),
                Err(err) => error!(?err, "Failed to get trading action"),
            }

            let holdings = gmgn_client
                .get_wallet_holdings(&self.wallet_address, None, None, None, None, None, None)
                .await;
            let prompt_analyze_holdings = format!(
            "Analyze my current portfolio holdings and provide your single best trading recommendation. Only give action, no explanation. \
            You may recommend holding if that's the best action. Your bag should buy only 3 tokens, no more. \
            \n\nIf recommending a trade, use one of these formats: \
            \n- For buying: `swap <amount> SOL to <token_address> (not symbol)` \
            \n- For selling: `swap <percentage>% <token_address> (not symbol) to SOL` \
            \n- For holding: Simply holding  \
            \n\nCurrent Holdings:\n{:?}",
            holdings);

            let response_analyze_holdings = match agent.prompt(&prompt_analyze_holdings).await {
                Ok(response) => response,
                Err(err) => {
                    error!(?err, "Failed to analyze holdings");
                    return;
                }
            };
            let action = agent.prompt(&response_analyze_holdings).await;
            match action {
                Ok(action_str) => debug!(action = %action_str, "Portfolio Action"),
                Err(err) => error!(?err, "Failed to get portfolio action"),
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(
                self.random_number(10 * 60, 60 * 60),
            )).await;        }

    }
    fn random_number(&self, min: u64, max: u64) -> u64 {
        let mut rng = rand::thread_rng();
        rng.gen_range(min..=max)
    }
}
