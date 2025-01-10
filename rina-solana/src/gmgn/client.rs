use reqwest;
use crate::gmgn::types::{TopHoldersResponse, HolderInfo, TokenInfoResponse, TokenInfo, WalletHoldingsResponse, WalletHoldingsData, SwapRankResponse};

const BASE_URL: &str = "https://gmgn.mobi";
pub struct GMGNClient {
    client: reqwest::Client,
}

impl GMGNClient {
    pub fn new() -> Self {
        let headers = {
            let mut headers = reqwest::header::HeaderMap::new();
            headers.insert("accept", "application/json, text/plain, */*".parse().unwrap());
            headers.insert("host", "gmgn.mobi".parse().unwrap());
            headers.insert("connection", "Keep-Alive".parse().unwrap());
            headers.insert("user-agent", "okhttp/4.9.2".parse().unwrap());
            headers
        };

        let client = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .unwrap();

        Self { client }
    }

    pub async fn get_top_holders(
        &self, 
        contract_address: &str,
        limit: Option<u32>,
        cost: Option<u32>,
        orderby: Option<&str>,
        direction: Option<&str>
    ) -> Result<Vec<HolderInfo>, reqwest::Error> {
        let limit = limit.unwrap_or(20);
        let cost = cost.unwrap_or(20);
        let orderby = orderby.unwrap_or("amount_percentage");
        let direction = direction.unwrap_or("desc");

        let url = format!(
            "{BASE_URL}/defi/quotation/v1/tokens/top_holders/sol/{contract_address}?limit={limit}&cost={cost}&orderby={orderby}&direction={direction}"
        );
        let response = self.client.get(url).send().await?;
        let top_holders_response: TopHoldersResponse = response.json().await?;
        Ok(top_holders_response.data)
    }

    pub async fn get_token_info(&self, contract_address: &str) -> Result<TokenInfo, reqwest::Error> {
        let url = format!(
            "{BASE_URL}/api/v1/token_info/sol/{contract_address}"
        );
        let response = self.client.get(url).send().await?;
        let token_info_response: TokenInfoResponse = response.json().await?;
        Ok(token_info_response.data)
    }

    pub async fn get_wallet_holdings(
        &self,
        wallet_address: &str,
        limit: Option<u32>,
        orderby: Option<&str>,
        direction: Option<&str>,
        showsmall: Option<bool>,
        sellout: Option<bool>,
        hide_abnormal: Option<bool>,
    ) -> Result<WalletHoldingsData, reqwest::Error> {
        let limit = limit.unwrap_or(50);
        let orderby = orderby.unwrap_or("last_active_timestamp");
        let direction = direction.unwrap_or("desc");
        let showsmall = showsmall.unwrap_or(false);
        let sellout = sellout.unwrap_or(false);
        let hide_abnormal = hide_abnormal.unwrap_or(false);

        let url = format!(
            "{BASE_URL}/api/v1/wallet_holdings/sol/{wallet_address}?limit={limit}&orderby={orderby}&direction={direction}&showsmall={showsmall}&sellout={sellout}&hide_abnormal={hide_abnormal}"
        );
        let response = self.client.get(url).send().await?;
        let holdings_response: WalletHoldingsResponse = response.json().await?;
        Ok(holdings_response.data)
    }

    pub async fn get_swap_rankings(
        &self, 
        time_period: &str, 
        launchpad: &str, 
        limit: Option<&str>,
    ) -> Result<SwapRankResponse, reqwest::Error> {
        let url = format!(
            "{BASE_URL}/defi/quotation/v1/rank/sol/swaps/{time_period}"
        );
        let params = vec![
            ("device_id", "1212e9167c96f7ee"),
            ("client_id", "gmgn_android_209000"), 
            ("from_app", "gmgn"),
            ("app_ver", "209000"),
            ("os", "android"),
            ("limit", limit.unwrap_or("20")),
            ("orderby", "marketcap"),
            ("direction", "desc"),
            ("filters[]", "renounced"),
            ("filters[]", "frozen")
        ];

        let response = self.client.get(url).query(&params).send().await?;
        let mut swap_rank_response: SwapRankResponse = response.json().await?;
        
        let launchpad = if launchpad.is_empty() { "Pump.fun" } else { launchpad };
        swap_rank_response.data.rank.retain(|token| token.launchpad.as_deref() == Some(launchpad));
        
        Ok(swap_rank_response)
    }
}
