# Reflex

Reflex is a Solana protocol developed with the Pinocchio framework, designed to create **incentive vaults** for existing Kalshi prediction markets using dFlow infrastructure. It empowers market participants to earn additional rewards by staking outcome tokens (YES/NO) from Kalshi markets, while ensuring robust security and performance through Pinocchio's optimized Solana program architecture.

## Overview

Reflex bridges prediction market liquidity with incentive mechanisms, providing:

- **Incentivized vaults** for Kalshi prediction market outcomes
- **Rewards for stakers** who deposit YES or NO outcome tokens into vaults
- **Proportional incentive distribution** to stakers upon market settlement
- **Support for bribers** to incentivize specific market outcomes

## Key Concepts

### Roles

- **Briber**: Initiates market vaults and deposits incentive tokens to reward stakers.
- **Farmer**: Stakes outcome tokens (YES/NO) to earn a share of the incentive pool.
- **Authority**: Administrative role managing protocol configuration and market settlement.

### Core Components

- **Market Vault**: Secure container for a specific Kalshi market's outcome tokens and incentives.
- **Farmer Position**: Tracks individual staker positions and rewards in a market vault.
- **Config**: Global protocol configuration managing fees and settings.

## Architecture

The protocol is built with the Pinocchio framework for optimized Solana program development, ensuring minimal runtime overhead, high security, and maintainability.

### Program Structure

```
src/
├── instructions/
│   ├── authority/     # Administrative operations (initialize, settle, update config)
│   ├── briber/        # Market vault creation, cancellation, incentive management
│   └── farmer/        # Staking and unstaking outcome tokens
├── states/            # On-chain account structures
│   ├── config.rs
│   ├── market_vault.rs
│   └── farmer_position.rs
└── lib.rs            # Program entrypoint
```

## Instructions

### Authority Instructions

- `Initialize`: Set up protocol configuration and parameters.
- `SettleMarket`: Finalize market resolution and enable reward claims.
- `UpdateConfig`: Modify protocol parameters and settings.
- `WithdrawTreasury`: Withdraw protocol fees.

### Briber Instructions

- `CreateMarketVault`: Create a new incentive vault for a Kalshi market.
- `AddIncentives`: Add additional incentive tokens to an existing vault.
- `CancelMarket`: Cancel a market vault before settlement.

### Farmer Instructions

- `StakeOutcomeToken`: Deposit YES or NO outcome tokens into a vault.
- `UnstakeOutcomeToken`: Withdraw outcome tokens and claim rewards.

## State Accounts

### Config

Stores global protocol settings:

- Authority address
- Market counter
- Protocol fee (basis points)
- Briber fee (basis points)

### MarketVault

Tracks a specific market's vault state:

- Briber address
- Outcome mint addresses (YES/NO)
- Incentive mint address
- Total staked amounts (YES/NO)
- Total incentives
- Fee accumulation
- Market status and resolution

### FarmerPosition

Tracks an individual farmer's position:

- Farmer address
- YES tokens staked
- NO tokens staked

## Fee Structure

The protocol implements a two-tier fee system:

1. **Staking Fees**: Applied when farmers stake outcome tokens.
   - Calculated as `amount × fee_bps / 10000`.
   - Reduces staked amount but does not affect farmer's share of incentives.

2. **Briber Fees**: Applied to bribers for vault creation and management.

## Build & Test

### Requirements

- Rust 1.70 or higher
- Solana CLI tools
- Anchor framework (optional, for testing)

### Build

```bash
cargo build-sbf
```

### Test

```bash
cargo test-sbf
```

## Development

This project is developed with the **Pinocchio** framework, ensuring lightweight, secure, and efficient Solana program development. Testing is performed using **Mollusk**.

Built with:

- **Pinocchio**: Lightweight Solana program framework
- **Solana SDK**: Core blockchain functionality
- **SPL Token**: Token program integration
- **Mollusk**: Testing and benchmarking framework

## Program ID

```
coming soon
```

## Security Considerations

- All arithmetic operations use checked math to prevent overflows.
- PDA derivations are validated on every instruction.
- Account ownership and signer checks are strictly enforced.
- Custom error types provide precise error handling.

## License

See LICENSE file for details.

## Contributing

Contributions are welcome! Please open an issue or pull request.
