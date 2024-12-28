use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct TransferArgs {
    pub token_type: String,
    pub recipient: String,
    pub amount: f64,
    pub token_mint: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum TransferError {
    #[error("Solana transfer error: {0}")]
    SolanaError(#[from] anyhow::Error),
    
    #[error("Missing token mint address for SPL transfer")]
    MissingTokenMint,
    
    #[error("Invalid token type, must be 'sol' or 'spl'")]
    InvalidTokenType,
}

#[derive(Debug, Deserialize)]
pub struct SwapArgs {
    pub input_mint: String,
    pub output_mint: String,
    pub amount: String,
    pub slippage_bps: Option<u32>,
}

#[derive(Debug, thiserror::Error)]
pub enum SwapError {
    #[error("Invalid mint address for {0}")]
    InvalidMintAddress(String),
    
    #[error("Jupiter error: {0}")]
    JupiterError(String),
    
    #[error("Failed to deserialize transaction: {0}")]
    TransactionDeserializeError(#[from] bincode::Error),
    
    #[error("Failed to sign transaction: {0}")]
    TransactionSignError(#[from] solana_sdk::signature::SignerError),
    
    #[error("Failed to submit transaction: {0}")]
    TransactionSubmitError(#[from] solana_client::client_error::ClientError),

    #[error("Insufficient token balance for swap")]
    InsufficientBalance,

    #[error("Invalid amount for swap")]
    InvalidAmount(String),
}