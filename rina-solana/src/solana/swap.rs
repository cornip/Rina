use anyhow::Result;
use jupiter_swap_api_client::{
    quote::{QuoteRequest},
    swap::SwapRequest,
    transaction_config::{TransactionConfig, ComputeUnitPriceMicroLamports},
    JupiterSwapApiClient,
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig, pubkey::Pubkey, signature::{Keypair, Signer}, transaction::VersionedTransaction
};
pub struct JupiterSwap {
    client: RpcClient,
    payer: Keypair,
    jupiter_client: JupiterSwapApiClient,
}

impl JupiterSwap {
    pub fn new(rpc_url: &str, private_key: &str) -> Result<Self> {
        let client =
            RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
        let api_base_url = "https://quote-api.jup.ag/v6";
        let payer = Keypair::from_base58_string(private_key);
        let jupiter_client = JupiterSwapApiClient::new(api_base_url.to_string());

        Ok(Self {
            client,
            payer,
            jupiter_client,
        })
    }

    pub async fn swap(
        &self,
        input_mint: Pubkey,
        output_mint: Pubkey,
        amount: u64,
        slippage_bps: u16,
        dexes: Option<Vec<String>>,
    ) -> Result<String> {
        // Get quote
        let quote_request = QuoteRequest {
            amount,
            input_mint,
            output_mint,
            slippage_bps,
            dexes,
            ..QuoteRequest::default()
        };

        let quote_response = self.jupiter_client.quote(&quote_request).await?;

        // Execute swap
        let priority_fee = 0.01 * solana_sdk::native_token::LAMPORTS_PER_SOL as f64;
        let swap_request = SwapRequest {
            user_public_key: self.payer.pubkey(),
            quote_response,
            config: TransactionConfig {
                compute_unit_price_micro_lamports: Some(ComputeUnitPriceMicroLamports::MicroLamports(priority_fee as u64)),
                ..TransactionConfig::default()
            },
        };
        let swap_response = self.jupiter_client.swap(&swap_request).await?;

        let versioned_transaction: VersionedTransaction =
            bincode::deserialize(&swap_response.swap_transaction)?;

        let signed_transaction =
            VersionedTransaction::try_new(versioned_transaction.message, &[&self.payer])?;

        let signature = self.client.send_transaction(&signed_transaction)?;

        Ok(signature.to_string())
    }

    pub fn get_token_balance(&self, token_mint: &Pubkey) -> Result<(f64, u64, u8)> {
        let associated_token_address = spl_associated_token_account::get_associated_token_address(
            &self.payer.pubkey(),
            token_mint,
        );
        let balance = self.client.get_token_account_balance(&associated_token_address)?;
        let decimals = balance.decimals;
        let raw_amount = balance.amount.parse::<u64>().unwrap_or(0);
        let float_amount = raw_amount as f64 / (10_f64.powi(decimals as i32));
        Ok((float_amount, raw_amount, decimals))
    }
    pub fn get_sol_balance(&self) -> Result<(f64, u64, u8)> {
        let balance = self.client.get_balance(&self.payer.pubkey())?;
        let decimals = 9;
        let raw_amount = balance;
        let float_amount = raw_amount as f64 / (10_f64.powi(decimals as i32));
        Ok((float_amount, raw_amount, decimals))
    }
}
