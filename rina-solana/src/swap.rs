use rig::{
    completion::ToolDefinition,
    tool::Tool,
};
use serde_json::json;
use crate::{
    solana::swap::JupiterSwap,
    types::{SwapArgs, SwapError},
};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use anyhow::Result;

pub struct SwapTool {
    jupiter_swap: JupiterSwap,
}

impl SwapTool {
    pub fn new() -> Self {
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .expect("SOLANA_RPC_URL not set");
        let private_key = std::env::var("SOLANA_PRIVATE_KEY")
            .expect("SOLANA_PRIVATE_KEY not set");
        
        Self {
            jupiter_swap: JupiterSwap::new(&rpc_url, &private_key)
                .expect("Failed to initialize Jupiter swap"),
        }
    }
}

impl Tool for SwapTool {
    const NAME: &'static str = "swap_tokens";
    
    type Error = SwapError;
    type Args = SwapArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "swap_tokens".to_string(),
            description: "Swap tokens using Jupiter Exchange on Solana".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "input_mint": {
                        "type": "string",
                        "description": "Input token mint address"
                    },
                    "output_mint": {
                        "type": "string",
                        "description": "Output token mint address"
                    },
                    "amount": {
                        "type": "string",
                        "description": "Amount to swap: use '%' suffix for percentage of balance (e.g., '50%'), or absolute amount (e.g., '1000')"
                    },
                    "slippage_bps": {
                        "type": "number",
                        "description": "Slippage tolerance in basis points (e.g., 50 = 0.5%)",
                        "default": 50
                    }
                },
                "required": ["input_mint", "output_mint", "amount"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        let input_mint = Pubkey::from_str(&args.input_mint)
            .map_err(|_| SwapError::InvalidMintAddress("input_mint".to_string()))?;
        let (_, raw_balance, decimals) = if &args.input_mint == "So11111111111111111111111111111111111111112" {
            self.jupiter_swap.get_sol_balance()
                .map_err(|e| SwapError::JupiterError(e.to_string()))?
        } else {
            self.jupiter_swap.get_token_balance(&input_mint)
                .map_err(|e| SwapError::JupiterError(e.to_string()))?
        };

        let raw_amount = if args.amount.ends_with('%') {
            let percentage = args.amount
                .trim_end_matches('%')
                .parse::<f64>()
                .map_err(|_| SwapError::InvalidAmount("Invalid percentage format".to_string()))?;
            
            if percentage <= 0.0 || percentage > 100.0 {
                return Err(SwapError::InvalidAmount("Percentage must be between 0 and 100".to_string()));
            }
            
            ((raw_balance as f64) * percentage / 100.0) as u64
        } else {
            (args.amount.parse::<f64>().unwrap() * (10_f64.powi(decimals as i32))) as u64
        };

        if raw_amount > raw_balance {
            return Err(SwapError::InsufficientBalance);
        }

        let output_mint = Pubkey::from_str(&args.output_mint)
            .map_err(|_| SwapError::InvalidMintAddress("output_mint".to_string()))?;
        
        self.jupiter_swap
            .swap(
                input_mint,
                output_mint,
                raw_amount,
                args.slippage_bps.unwrap_or(50) as u16,
                None,
            )
            .await
            .map_err(|e| SwapError::JupiterError(e.to_string()))
    }
}
