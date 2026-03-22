# Reflex

**Reflex** is a Solana on-chain program built with [Pinocchio](https://github.com/anza-xyz/pinocchio) that implements an incentive vault system for [Kalshi](https://kalshi.com/) prediction markets, running on [DFlow](https://dflow.net/)'s infrastructure. It allows any party (a _briber_) to bootstrap liquidity on a prediction market by depositing token incentive rewards that are distributed to participants (_farmers_) who take a side and stake their outcome tokens.

[Reflex client repository](https://github.com/franRappazzini/reflex-client)

---

## Why Reflex?

Prediction markets have a cold-start problem: participants are reluctant to stake on a market that has no liquidity, and liquidity providers are reluctant to enter a market with no participants. Reflex breaks this loop by creating an economic incentive layer on top of existing prediction market outcome tokens:

- **Bribers** (protocols, DAOs, market-makers, or any stakeholder with interest in a market having deep liquidity) deposit a reward pool denominated in WSOL or USDC.
- **Farmers** stake their YES or NO outcome tokens to earn a proportional share of that reward pool.
- Once the market settles, farmers on the winning side claim their staked tokens back plus their share of the incentive pool. Bribers collect the staking fees that accumulated on the winning side.

This makes it economically attractive to provide outcome token liquidity on Kalshi markets, deepening the order book and improving price discovery for everyone.

**Why Pinocchio?** Unlike Anchor, Pinocchio operates in `no_std` without a heap allocator and manipulates account memory directly via raw pointers. This results in dramatically smaller binary size and lower compute unit consumption per instruction — critical for a high-throughput protocol that expects many concurrent stake/unstake operations.

---

## Architecture Overview

The protocol has three actors and five on-chain account types:

```
Authority ──► manages global config, settles markets, withdraws protocol fees
Briber    ──► creates markets with incentive deposits, adds rewards, claims staking fees
Farmer    ──► stakes outcome tokens to earn incentive rewards
```

### On-chain Accounts

| Account          | Type                      | Seeds                                 |
| ---------------- | ------------------------- | ------------------------------------- |
| `Config`         | Protocol config           | `["config"]`                          |
| `Treasury`       | Token account (WSOL/USDC) | `["treasury", mint]`                  |
| `Market`         | Per-market state          | `["market", id]`                      |
| `Market Vault`   | Token account per vault   | `["market", market_addr, mint]`       |
| `FarmerPosition` | Per-farmer-per-market     | `["farmer_position", market, farmer]` |

Each market has three vaults: one for the incentive tokens (`incentive_vault`), and one each for staked YES and NO outcome tokens (`yes_vault`, `no_vault`). Staking fees accumulate inside the outcome vaults and are claimable by the briber after settlement.

### Instruction Set

| #   | Instruction           | Actor     |
| --- | --------------------- | --------- |
| 0   | `Initialize`          | Authority |
| 1   | `CreateMarket`        | Briber    |
| 2   | `AddIncentives`       | Briber    |
| 3   | `CancelMarket`        | Briber    |
| 4   | `ClaimFees`           | Briber    |
| 5   | `SettleMarket`        | Authority |
| 6   | `WithdrawTreasury`    | Authority |
| 7   | `StakeOutcomeToken`   | Farmer    |
| 8   | `UnstakeOutcomeToken` | Farmer    |
| 9   | `ClaimRewards`        | Farmer    |
| 10  | `UpdateConfig`        | Authority |

### Reward Formula

$$\text{reward} = \frac{\text{farmer\_staked} \times \text{total\_incentives}}{\text{total\_winning\_staked}}$$

### Fee Model

- **Protocol fee** (`fee_bps`): charged to the briber on every `CreateMarket` / `AddIncentives` call. Flows to the protocol treasury (WSOL or USDC).
- **Staking fee** (`briber_fee_bps`): charged to farmers on `StakeOutcomeToken`. Accumulates inside the winning outcome vault and is claimable by the briber after settlement via `ClaimFees`.

Both fees are expressed in basis points (1 bps = 0.01%). Maximum value for each is 5 000 bps (50%).

---

## Program ID

```
4ZegtDo8WG6e2PAswLhnGXYDS5TGkniVCKXDrDX12KYX
```

---

## Prerequisites

| Tool                                                               | Version               | Purpose                                        |
| ------------------------------------------------------------------ | --------------------- | ---------------------------------------------- |
| [Rust](https://rustup.rs/)                                         | stable (2024 edition) | Build the program                              |
| [Solana CLI](https://docs.solana.com/cli/install-solana-cli-tools) | ≥ 2.x                 | `cargo build-sbf`, keypair management          |
| [Node.js](https://nodejs.org/)                                     | ≥ 20                  | Run the TypeScript test suite                  |
| [pnpm](https://pnpm.io/)                                           | ≥ 9                   | Node package manager                           |
| [Surfpool](https://github.com/txtx/surfpool)                       | latest                | Local Solana network for development & testing |
| [txtx](https://txtx.sh/)                                           | latest                | IaC deployment tool for Solana                 |

---

## Getting Started

### 1. Clone & install dependencies

```bash
git clone <repo-url>
cd reflex
pnpm install
```

### 2. Build the program

```bash
cargo build-sbf
```

The compiled `.so` binary will be placed at `target/deploy/reflex.so`.

### 3. Start Surfpool (local network)

Surfpool provides a local Solana-compatible RPC with instant block finality, ideal for testing.

```bash
surfpool start
```

By default it listens on `http://127.0.0.1:8899`. Keep this process running in a separate terminal.

### 4. Run the test suite

```bash
pnpm test
```

This builds the program, starts Surfpool, deploys the program, and runs all 11 end-to-end instruction tests in [tests/reflex.test.ts](tests/reflex.test.ts) covering the full lifecycle: initialize → create market → add incentives → stake → settle → claim fees → claim rewards → withdraw treasury.

---

## Project Structure

```
src/
├── lib.rs                        # Program entrypoint & instruction dispatcher
├── instructions/
│   ├── authority/                # Initialize, SettleMarket, UpdateConfig, WithdrawTreasury
│   ├── briber/                   # CreateMarket, AddIncentives, CancelMarket, ClaimFees
│   └── farmer/                   # StakeOutcomeToken, UnstakeOutcomeToken, ClaimRewards
├── states/
│   ├── config.rs                 # Config PDA (37 bytes)
│   ├── market.rs                 # Market PDA (181 bytes)
│   └── farmer_position.rs        # FarmerPosition PDA (18 bytes)
└── utils/
    ├── constants.rs              # WSOL/USDC addresses, PDA seeds
    ├── math.rs                   # Fee calculation (u128-safe)
    └── helpers/
        ├── account.rs            # PDA creation, lazy-init, close helpers
        └── token.rs              # SPL Token / Token-2022 CPI wrappers

tests/
├── reflex.test.ts                # Full end-to-end test suite
└── instructions/                 # Per-instruction test helpers
runbooks/
└── deployment/                   # txtx IaC deployment runbook
```

---

## Supported Token Standards

- **SPL Token** (legacy)
- **Token-2022** — auto-detected by owner and account discriminator at offset 165.

Only **WSOL** and **USDC** are accepted as incentive mints and treasury mints.

---

## Security Notes

- All PDAs are validated by re-deriving seeds and comparing with the provided account address on every instruction.
- Authority checks are enforced on every privileged instruction (`Initialize`, `SettleMarket`, `UpdateConfig`, `WithdrawTreasury`).
- `CancelMarket` is blocked if any staking fees have been collected (non-zero `available_yes_fees` or `available_no_fees`) to protect farmers from rug pulls.
- Reward arithmetic uses 128-bit intermediate values to prevent overflow.
- The program compiles with `no_std`, no allocator, and no unsafe heap usage beyond Pinocchio's account memory access model.

---

## License

MIT
