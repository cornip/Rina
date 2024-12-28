use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    message::Message,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
    compute_budget::ComputeBudgetInstruction,
};
use spl_token::instruction as spl_instruction;
use std::str::FromStr;
use anyhow::Result;
use spl_associated_token_account::get_associated_token_address;
use spl_associated_token_account::instruction::create_associated_token_account;

pub struct SolanaTransfer {
    client: RpcClient,
    payer: Keypair,
}

impl SolanaTransfer {
    pub fn new(rpc_url: &str, private_key: &str) -> Result<Self> {
        let client = RpcClient::new_with_commitment(
            rpc_url.to_string(),
            CommitmentConfig::confirmed(),
        );

        // Convert private key from base58 to Keypair
        let payer = Keypair::from_base58_string(private_key);

        Ok(Self { client, payer })
    }

    fn send_transaction_with_priority(&self, instructions: &[solana_sdk::instruction::Instruction], priority_fee: u64) -> Result<String> {
        let recent_blockhash = self.client.get_latest_blockhash()?;
        let priority_instruction = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
        let mut all_instructions = vec![priority_instruction];
        all_instructions.extend_from_slice(instructions);

        let message = Message::new(&all_instructions, Some(&self.payer.pubkey()));
        let transaction = Transaction::new(&[&self.payer], message, recent_blockhash);

        let signature = self.client.send_transaction(&transaction)?;
        Ok(signature.to_string())
    }

    pub async fn transfer_sol(&self, to_pubkey: &str, amount_sol: f64) -> Result<String> {
        let to_pubkey = Pubkey::from_str(to_pubkey)?;
        let amount_lamports = (amount_sol * 1_000_000_000.0) as u64; // Convert SOL to lamports

        let instruction = system_instruction::transfer(
            &self.payer.pubkey(),
            &to_pubkey,
            amount_lamports,
        );
        let priority_fee = 100_000;
        self.send_transaction_with_priority(&[instruction], priority_fee)
    }

    pub async fn transfer_spl(
        &self,
        token_mint: &str,
        to_address: &str,
        amount: u64,
    ) -> Result<String> {
        let token_mint_pubkey = Pubkey::from_str(token_mint)?;
        let from_token_account = get_associated_token_address(&self.payer.pubkey(), &token_mint_pubkey);
        let balance = self.client.get_token_account_balance(&from_token_account)?;
        
        // Ensure sender's token account exists
        if self.client.get_account(&from_token_account).is_err() {
            return Err(anyhow::anyhow!("Sender's token account does not exist"));
        }

        // Parse recipient address and get token account
        let to_wallet = Pubkey::from_str(to_address)?;
        let to_token_account = get_associated_token_address(&to_wallet, &token_mint_pubkey);
        

        let mut instructions = vec![];
        
        // Create ATA for recipient if it doesn't exist
        if self.client.get_account(&to_token_account).is_err() {
            instructions.push(
                create_associated_token_account(
                    &self.payer.pubkey(),
                    &to_wallet,
                    &token_mint_pubkey,
                    &spl_token::id(),
                )
            );
        }
        let decimals = balance.decimals;
        let raw_amount = (amount as f64 * (10_f64.powi(decimals as i32))) as u64;
        instructions.push(
            spl_instruction::transfer(
                &spl_token::id(),
                &from_token_account,
                &to_token_account,
                &self.payer.pubkey(),
                &[&self.payer.pubkey()],
                raw_amount,
            )?
        );
        let priority_fee = 100_000;
        self.send_transaction_with_priority(&instructions, priority_fee)
    }
} 