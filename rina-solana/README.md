# Solana Token Assistant

A command-line tool for interacting with Solana tokens using natural language. Powered by GPT-4, this assistant can help you perform token swaps via Jupiter, and transfer SOL or SPL tokens.

## Features

- 🔄 Token swaps using Jupiter Exchange (Pumpfun, Raydium both supported)
- 💸 SOL and SPL token transfers

## Prerequisites

- Rust and Cargo installed
- OpenAI API key
- Solana RPC URL (e.g., Helius)
- Solana wallet private key

## Setup

1. Clone the repository

### Supported Operations

```bash
git clone https://github.com/cornip/rig-solana-token-assistant
cd rig-solana-token-assistant
```

2. Create a `.env` file with your API keys:

```env
OPENAI_API_KEY=
SOLANA_RPC_URL=
SOLANA_PRIVATE_KEY=
```

3. Build and run:

```bash
cargo run
```

## Usage

After starting the program, you can interact with it using natural language commands:

1. **Token Swaps**
   - Swap between any SPL tokens using Jupiter Exchange
   - Example: "Swap 1 SOL to USDC"
   - Example: "Swap 1 SOL to 6wUfdjiBtXjiWTfwGabBqybVTCAFoS9iD3X6t9v1pump"
2. **Token Transfers**
   - Transfer SOL or SPL tokens to any address
   - Example: "Transfer 0.5 SOL to address ABC..."
   - Example: "Transfer 10 USDC to address ABC..."
   - Example: "Transfer 6wUfdjiBtXjiWTfwGabBqybVTCAFoS9iD3X6t9v1pump to address ABC..."

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
