# Emblem Vault SDK

TypeScript SDK for [Emblem Vault](https://emblemvault.ai) cross-chain wallet and trading infrastructure.

Wraps the Emblem Vault API for wallet management, token swaps, cross-chain bridges, and market data across 7 blockchains.

## Install

```bash
npm install emblem-vault-sdk
```

## Usage

```typescript
import { EmblemClient, Chain } from 'emblem-vault-sdk';

const client = new EmblemClient({
  apiKey: process.env.EMBLEM_API_KEY,
  walletPassword: process.env.EMBLEM_WALLET_PASSWORD,
});

// Get wallet balances
const balances = await client.getBalances(Chain.SOLANA);

// Swap tokens
const result = await client.swap({
  chain: Chain.SOLANA,
  fromToken: 'SOL',
  toToken: 'USDC',
  amount: 1.0,
  slippageBps: 100,
});

// Bridge tokens
const bridge = await client.bridge({
  fromChain: Chain.SOLANA,
  toChain: Chain.ETHEREUM,
  token: 'USDC',
  amount: 100,
});

// Get token price
const price = await client.getPrice('SOL', Chain.SOLANA);
```

## Supported Chains

Solana, Ethereum, Base, BSC, Polygon, Hedera, Bitcoin.

## Features

- Multi-chain wallet management
- Token swaps via Jupiter, Uniswap, PancakeSwap
- Cross-chain bridging
- Market data and token lookup
- Limit orders and conditional trades
- Balance monitoring and alerts

## Links

- Platform: [emblemvault.ai](https://emblemvault.ai)
- Docs: [docs.emblem.wiki](https://docs.emblem.wiki)
- X: [@EmblemVault](https://x.com/EmblemVault)

## License

MIT
