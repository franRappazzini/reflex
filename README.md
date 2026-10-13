# Reflex

Reflex is a Solana program for adding transparent, time-weighted incentives to
binary prediction markets. A market has YES and NO outcome tokens and one
shared incentive pool. Participants stake either outcome; the program tracks
stake-seconds on each side and, after settlement, distributes the pool to
positions on the winning side in proportion to their accumulated stake-time.
This makes the incentive pool finite and ties its distribution to the market's
actual on-chain resolution instead of an estimated event duration.

The protocol is designed to compose with tokenized prediction markets,
including markets built with World infrastructure. Reflex manages incentives
and outcome-token custody; it does not create or resolve the underlying
prediction market. <!-- See the [Reflex whitepaper](./reflex%20whitepaper.pdf) for
the protocol motivation and economic model. -->

## This repository

This is an Anchor workspace containing the Reflex Solana program, Rust
integration tests, and Surfpool deployment runbooks.

<!-- - **Program ID:** `8i3X3C7WaRGxKLTpoecUqmPEueMULXVTRzEJFuyGrTHd` -->

- **Program source:** [`programs/reflex/src`](./programs/reflex/src)
- **Integration tests:** [`tests/src`](./tests/src)
- **Local test script:** [`scripts/test.sh`](./scripts/test.sh)
- **Deployment runbook:** [`runbooks/deployment`](./runbooks/deployment)

### Instructions

| Instruction                           | Purpose                                                                                                           |
| ------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `initialize(fee_bps, briber_fee_bps)` | Creates the global configuration PDA and sets its authority and fee parameters.                                   |
| `create_pool(amount)`                 | Creates a pool for a market, initializes its token vaults, and deposits the initial incentive amount.             |
| `add_incentives(amount)`              | Adds tokens to an existing pool's incentive vault. Currently callable by the pool creator.                        |
| `stake_outcome(amount)`               | Stakes YES or NO outcome tokens while the pool is open and begins accumulating stake-time for that side.          |
| `unstake_outcome(amount)`             | Withdraws staked outcome tokens while the pool is open.                                                           |
| `settle_pool(decision)`               | Records the market outcome and freezes the stake-time accumulators. Restricted to the configured authority.       |
| `claim_rewards()`                     | Lets a position on the winning side claim its staked outcome tokens and stake-time-proportional incentive payout. |
| `collect_fees()`                      | Lets the pool creator collect fees for the winning side after settlement.                                         |

### On-chain accounts

- **Config PDA:** global authority and fee settings.
- **Pool PDA:** one per market; stores outcome mints, incentive totals, side-specific
  stake-time accumulators, fees, and settlement status.
- **Position PDA:** one per staker and pool; tracks that staker's YES and NO
  stake and accumulated stake-time.
- **Vault PDAs:** hold the incentive tokens and the pool's YES/NO outcome tokens.

The program uses Anchor and SPL Token interfaces. Market accounts and outcome
mints are supplied to the instructions; the market's own creation and
resolution are outside this program.

## Requirements

- Rust toolchain **1.89.0** (selected by [`rust-toolchain.toml`](./rust-toolchain.toml))
- Solana CLI / local Solana runtime
- Anchor CLI compatible with the Anchor 1.2.0 dependencies in this workspace
- Surfpool for the local integration-test workflow

The integration tests connect to the local RPC at `http://127.0.0.1:8899` and
use Surfpool's `surfnet_timeTravel` RPC method. They also expect:

- `ANCHOR_WALLET` to point to the authority keypair (defaults in Anchor config
  to `~/.config/solana/id.json`)
- `tests/keypairs/creator.json` and `tests/keypairs/staker.json`

Those two test keypairs are ignored by Git and are not included in the
repository; provide local test-only keypairs before running the integration
tests.

## Build and test locally

Build the program with the feature used by the integration-test setup:

```sh
anchor build -- --features local-test
```

In one terminal, start a local Surfnet (it will also start a local Solana validator and deploy the program):

```sh
surfpool start
```

Then run the integration tests from the repository
root in another terminal:

```sh
anchor test --skip-local-validator --skip-build --skip-deploy
```

The lifecycle tests exercise initialization, pool creation, staking,
unstaking, incentive deposits, settlement, reward claims, and fee collection.

To use the Surfpool watch workflow instead, see the
[runbooks guide](./runbooks/README.md).

## Next features

- 👨🏻‍💻 — Open incentive funding: allow anyone to contribute to a pool,
  rather than limiting contributions to its creator.
- ⬜ — Complete World integration: automate market/pool creation, settlement,
  and the end-to-end lifecycle, including claiming market winnings and
  collecting fees through World.
- ⬜ — Rework staker positions: replace per-staker Position PDAs with an
  SPL-token-based position flow.
- ⬜ — Add Kamino integration so incentive funders can earn yield on
  deposited funds.
<!-- - **Finalize reward and fee accounting:** complete normalization and fee
  handling in reward claims, with tests for precision, rounding, and pool
  edge cases.
- **Harden and test the integrated lifecycle:** validate World market state
  and exercise creation, resolution, claims, and failure paths end to end. -->

## Repository layout

```text
programs/reflex/       Anchor program and instruction/account definitions
tests/                 Rust integration tests and local test helpers
scripts/test.sh        Anchor test command used by the repository
runbooks/deployment/   Surfpool deployment configuration
Anchor.toml            Anchor program ID, localnet provider, and test settings
```
