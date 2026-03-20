import {
  MarketResolution,
  MarketStatus,
  fetchMarket,
  fetchMaybeMarket,
} from "./utils/fetch/market";
import {
  TOKEN_PROGRAM_ADDRESS,
  findAssociatedTokenPda,
  getCreateAssociatedTokenInstructionAsync,
  getMintToInstruction,
} from "@solana-program/token";
import { fetchFarmerPosition, fetchMaybeFarmerPosition } from "./utils/fetch/farmer_position";
import { getConfigPda, getFarmerPositionPda, getMarketPda } from "./utils/pda";

import { KeyPairSigner } from "@solana/kit";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { buildAddIncentivesIx } from "./instructions/add_incentives";
import { buildAndSendTransaction } from "./utils/tx";
import { buildCancelMarketIx } from "./instructions/cancel_market";
import { buildClaimFeesIxs } from "./instructions/claim_fees";
import { buildClaimRewardsIxs } from "./instructions/claim_rewards";
import { buildCreateMarketIxs } from "./instructions/create_market";
import { buildInitializeIx } from "./instructions/initialize";
import { buildSettleMarketIx } from "./instructions/settle_market";
import { buildStakeOutcomeTokenIx } from "./instructions/stake_outcome_token";
import { buildUnstakeOutcomeTokenIx } from "./instructions/unstake_outcome_token";
import { buildUpdateConfigIx } from "./instructions/update_config";
import { buildWithdrawTreasuryIxs } from "./instructions/withdraw_treasury";
import { constants } from "./utils/constants";
import { createAccounts } from "./utils/accounts";
import { createClient } from "./utils/client";
import { createMint } from "./utils/mint";
import { expect } from "chai";
import { fetchConfig } from "./utils/fetch/config";

describe("reflex", () => {
  let client: Awaited<ReturnType<typeof createClient>>;
  let accounts: Awaited<ReturnType<typeof createAccounts>>;
  let yesMint: KeyPairSigner;
  let noMint: KeyPairSigner;

  before(async () => {
    client = await createClient();
    accounts = await createAccounts(client);
    [yesMint, noMint] = await Promise.all([createMint(client), createMint(client)]);
  });

  it("--- initialize ix ---", async () => {
    const feeBps = 500;
    const briberFeeBps = 500;

    const ix = await buildInitializeIx(client, feeBps, briberFeeBps);
    const txSig = await buildAndSendTransaction(client, [ix]);
    console.log("initialize tx:", txSig);

    const configAddress = await getConfigPda();
    const config = await fetchConfig(client.rpc, configAddress);

    expect(config.authority).to.equal(client.wallet.address);
    expect(config.feeBps).to.equal(feeBps);
    expect(config.briberFeeBps).to.equal(briberFeeBps);
  });

  it("--- update_config ix ---", async () => {
    const newFeeBps = 300;
    const newBriberFeeBps = 200;

    const configAddress = await getConfigPda();
    const configBefore = await fetchConfig(client.rpc, configAddress);

    const ix = await buildUpdateConfigIx(client, {
      newAuthority: client.wallet.address,
      newFeeBps,
      newBriberFeeBps,
    });
    const txSig = await buildAndSendTransaction(client, [ix]);
    console.log("update_config tx:", txSig);

    const configAfter = await fetchConfig(client.rpc, configAddress);
    expect(configAfter.authority).to.equal(client.wallet.address);
    expect(configAfter.feeBps).to.equal(newFeeBps);
    expect(configAfter.briberFeeBps).to.equal(newBriberFeeBps);
    // bump must not change
    expect(configAfter.bump).to.equal(configBefore.bump);
  });

  it.skip("--- create_market ix ---", async () => {
    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";
    const amount = BigInt(10 * LAMPORTS_PER_SOL);

    const ixs = await buildCreateMarketIxs(client, accounts, {
      id,
      amount,
      yesMint,
      noMint,
    });

    const txSig = await buildAndSendTransaction(client, ixs, {
      feePayer: accounts.briber,
      additionalSigners: [client.wallet],
    });
    console.log("create_market tx:", txSig);

    const marketAddress = await getMarketPda(id);
    const market = await fetchMarket(client.rpc, marketAddress);

    expect(market.briber).to.equal(accounts.briber.address);
    expect(market.totalIncentiveAmount).to.equal(amount);
    expect(market.incentiveMint).to.equal(constants.WSOL_MINT);
  });

  it.skip("--- add_incentives ix ---", async () => {
    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";
    const amount = BigInt(5 * LAMPORTS_PER_SOL);

    const marketAddress = await getMarketPda(id);
    const marketBefore = await fetchMarket(client.rpc, marketAddress);

    const ix = await buildAddIncentivesIx(accounts, { id, amount });

    const txSig = await buildAndSendTransaction(client, [ix], {
      feePayer: accounts.briber,
    });
    console.log("add_incentives tx:", txSig);

    const market = await fetchMarket(client.rpc, marketAddress);
    expect(market.totalIncentiveAmount).to.equal(marketBefore.totalIncentiveAmount + amount);
  });

  it.skip("--- cancel_market ix ---", async () => {
    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";

    // cancel_market transfers all incentives back to the briber and closes
    // market + vault accounts. It requires no pending fees on the market.
    const ix = await buildCancelMarketIx(accounts, {
      id,
      yesMint: yesMint.address,
      noMint: noMint.address,
    });

    const txSig = await buildAndSendTransaction(client, [ix], {
      feePayer: accounts.briber,
    });
    console.log("cancel_market tx:", txSig);

    const marketAddress = await getMarketPda(id);
    const market = await fetchMaybeMarket(client.rpc, marketAddress);
    expect(market).to.be.null;
  });

  it.skip("--- stake_outcome_token ix ---", async () => {
    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";
    const stakeAmount = BigInt(100_000_000); // 100 token (6 decimals)

    // fetch market to get the yes mint
    const marketAddress = await getMarketPda(id);
    const market = await fetchMarket(client.rpc, marketAddress);
    const outcomeMint = market.outcomeYesMint;

    // create farmer ATA for yes mint and mint tokens into it
    const [[farmerAta]] = await Promise.all([
      findAssociatedTokenPda({
        mint: outcomeMint,
        owner: accounts.farmer.address,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
      }),
    ]);

    const createFarmerAtaIx = await getCreateAssociatedTokenInstructionAsync({
      payer: client.wallet,
      ata: farmerAta,
      owner: accounts.farmer.address,
      mint: outcomeMint,
    });

    const mintToIx = getMintToInstruction({
      mint: outcomeMint,
      token: farmerAta,
      mintAuthority: client.wallet,
      amount: stakeAmount,
    });

    await buildAndSendTransaction(client, [createFarmerAtaIx, mintToIx]);

    // stake
    const ix = await buildStakeOutcomeTokenIx({
      id,
      amount: stakeAmount,
      outcomeMint,
      farmer: accounts.farmer,
    });

    const txSig = await buildAndSendTransaction(client, [ix], {
      additionalSigners: [accounts.farmer],
    });
    console.log("stake_outcome_token tx:", txSig);

    const marketAfter = await fetchMarket(client.rpc, marketAddress);
    expect(marketAfter.totalYesStaked > 0n).to.be.true;
    expect(marketAfter.availableYesFees > 0n).to.be.true;
    expect(marketAfter.totalNoStaked).to.equal(0n);
    expect(marketAfter.availableNoFees).to.equal(0n);
  });

  it.skip("--- unstake_outcome_token ix ---", async () => {
    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";

    const marketAddress = await getMarketPda(id);
    const market = await fetchMarket(client.rpc, marketAddress);
    const outcomeMint = market.outcomeYesMint;

    const farmerPositionAddress = await getFarmerPositionPda(
      marketAddress,
      accounts.farmer.address,
    );
    const positionBefore = await fetchFarmerPosition(client.rpc, farmerPositionAddress);

    // unstake half of what this farmer has staked
    const unstakeAmount = positionBefore.yesStaked / 2n;

    const ix = await buildUnstakeOutcomeTokenIx({
      id,
      amount: unstakeAmount,
      outcomeMint,
      farmer: accounts.farmer,
    });

    const txSig = await buildAndSendTransaction(client, [ix], {
      additionalSigners: [accounts.farmer],
    });
    console.log("unstake_outcome_token tx:", txSig);

    const [marketAfter, positionAfter] = await Promise.all([
      fetchMarket(client.rpc, marketAddress),
      fetchFarmerPosition(client.rpc, farmerPositionAddress),
    ]);

    // market totals decreased by the unstaked amount
    expect(marketAfter.totalYesStaked).to.equal(market.totalYesStaked - unstakeAmount);
    // fees are unaffected by unstaking
    expect(marketAfter.availableYesFees).to.equal(market.availableYesFees);
    // farmer position reflects the change
    expect(positionAfter.yesStaked).to.equal(positionBefore.yesStaked - unstakeAmount);
    expect(positionAfter.noStaked).to.equal(0n);
  });

  it.skip("--- settle_market ix ---", async () => {
    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";

    const ix = await buildSettleMarketIx(client, {
      id,
      resolution: MarketResolution.Yes,
    });

    const txSig = await buildAndSendTransaction(client, [ix]);
    console.log("settle_market tx:", txSig);

    const marketAddress = await getMarketPda(id);
    const market = await fetchMarket(client.rpc, marketAddress);

    expect(market.resolution).to.equal(MarketResolution.Yes);
    expect(market.status).to.equal(MarketStatus.Settled);
  });

  // claim_fees requires a settled market. A fresh market is created here since
  // the main one was already cancelled. It is settled inline before claiming.
  it.skip("--- claim_fees ix ---", async () => {
    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";

    // read on-chain state to pick the correct outcome mint.
    const marketAddress = await getMarketPda(id);
    const market = await fetchMarket(client.rpc, marketAddress);
    const outcomeMint =
      market.resolution === MarketResolution.Yes ? market.outcomeYesMint : market.outcomeNoMint;

    // claim fees.
    const ixs = await buildClaimFeesIxs(accounts, { id, outcomeMint });
    const txSig = await buildAndSendTransaction(client, ixs, {
      feePayer: accounts.briber,
    });
    console.log("claim_fees tx:", txSig);

    // after claiming, available fees must be 0.
    const marketAfter = await fetchMarket(client.rpc, marketAddress);
    expect(marketAfter.availableYesFees).to.equal(0n);
    expect(marketAfter.availableNoFees).to.equal(0n);
  });

  it.skip("--- claim_rewards ix ---", async () => {
    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";

    const marketAddress = await getMarketPda(id);
    const market = await fetchMarket(client.rpc, marketAddress);
    // market is resolved Yes — reward_mint is the incentive mint (WSOL),
    // outcome_mint is the winning side.
    const outcomeMint = market.outcomeYesMint;
    const rewardMint = market.incentiveMint;

    const farmerPositionAddress = await getFarmerPositionPda(
      marketAddress,
      accounts.farmer.address,
    );
    const positionBefore = await fetchFarmerPosition(client.rpc, farmerPositionAddress);

    const ixs = await buildClaimRewardsIxs({
      id,
      rewardMint,
      outcomeMint,
      farmer: accounts.farmer,
    });

    const txSig = await buildAndSendTransaction(client, ixs, {
      additionalSigners: [accounts.farmer],
    });
    console.log("claim_rewards tx:", txSig);

    // farmer_position must be closed after claiming
    const positionAfter = await fetchMaybeFarmerPosition(client.rpc, farmerPositionAddress);
    expect(positionAfter).to.be.null;

    // the staked amount was non-zero before claiming
    expect(positionBefore.yesStaked > 0n).to.be.true;
  });

  it.skip("--- withdraw_treasury ix ---", async () => {
    // treasury PDAs into them.
    const ixs = await buildWithdrawTreasuryIxs(client);
    const txSig = await buildAndSendTransaction(client, ixs);
    console.log("withdraw_treasury tx:", txSig);
  });
});
