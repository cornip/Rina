use rig::{
    completion::ToolDefinition,
    tool::Tool,
};
use serde_json::json;

use crate::{
    types::{TransferArgs, TransferError},
    solana::transfer::SolanaTransfer,
};

pub struct TransferTool {
    solana: SolanaTransfer,
}

impl TransferTool {
    pub fn new(rpc_url: &str, private_key: &str) -> Result<Self, anyhow::Error> {
        Ok(Self {
            solana: SolanaTransfer::new(rpc_url, private_key)?
        })
    }

    pub fn from_env() -> Result<Self, anyhow::Error> {
        let rpc_url = std::env::var("SOLANA_RPC_URL")
            .map_err(|_| anyhow::anyhow!("SOLANA_RPC_URL not set"))?;
        let private_key = std::env::var("SOLANA_PRIVATE_KEY")
            .map_err(|_| anyhow::anyhow!("SOLANA_PRIVATE_KEY not set"))?;
        
        Self::new(&rpc_url, &private_key)
    }
}

impl Tool for TransferTool {
    const NAME: &'static str = "transfer_tokens";
    
    type Error = TransferError;
    type Args = TransferArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: "transfer_tokens".to_string(),
            description: "Transfer SOL or SPL tokens on Solana".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "token_type": {
                        "type": "string",
                        "description": "Type of token to transfer ('sol' or 'spl')",
                        "enum": ["sol", "spl"]
                    },
                    "recipient": {
                        "type": "string",
                        "description": "Recipient's address (wallet for SOL, token account for SPL)"
                    },
                    "amount": {
                        "type": "number",
                        "description": "Amount to transfer (in SOL for SOL transfers, raw amount for SPL)"
                    },
                    "token_mint": {
                        "type": "string",
                        "description": "Required for SPL: Token mint address"
                    }
                },
                "required": ["token_type", "recipient", "amount"],
                "if": {
                    "properties": {
                        "token_type": { "const": "spl" }
                    }
                },
                "then": {
                    "required": ["token_mint", "from_token_account"]
                }
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        match args.token_type.as_str() {
            "sol" => {
                self.solana
                    .transfer_sol(&args.recipient, args.amount)
                    .await
                    .map_err(TransferError::SolanaError)
            }
            "spl" => {
                let token_mint = args.token_mint.ok_or(TransferError::MissingTokenMint)?;
                
                self.solana
                    .transfer_spl(
                        &token_mint,
                        &args.recipient,
                        args.amount as u64,
                    )
                    .await
                    .map_err(TransferError::SolanaError)
            }
            _ => Err(TransferError::InvalidTokenType),
        }
    }
}
