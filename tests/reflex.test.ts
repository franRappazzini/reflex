import {
  KeyPairSigner,
  SOLANA_ERROR__ACCOUNTS__ACCOUNT_NOT_FOUND,
  SolanaError,
  SolanaErrorCode,
} from "@solana/kit";
import { MarketResolution, MarketStatus, fetchMarket } from "./utils/fetch/market";
import { getConfigPda, getMarketPda } from "./utils/pda";

import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { buildAddIncentivesIx } from "./instructions/add_incentives";
import { buildAndSendTransaction } from "./utils/tx";
import { buildCancelMarketIx } from "./instructions/cancel_market";
import { buildClaimFeesIx } from "./instructions/claim_fees";
import { buildCreateMarketIxs } from "./instructions/create_market";
import { buildInitializeIx } from "./instructions/initialize";
import { buildSettleMarketIx } from "./instructions/settle_market";
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

  it("--- create_market ix ---", async () => {
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

  it("--- add_incentives ix ---", async () => {
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
    try {
      await fetchMarket(client.rpc, marketAddress);
      expect.fail(
        "Expected fetchMarket to throw an error since the market should have been cancelled and closed.",
      );
    } catch (err) {
      if (err instanceof SolanaError) {
        expect(err.message).to.include("Account not found");
      } else {
        expect.fail(`Expected a SolanaError with 'Account not found', got ${err}`);
      }
    }
  });

  it("--- settle_market ix ---", async () => {
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
  it("--- claim_fees ix ---", async () => {
    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";

    // read on-chain state to pick the correct outcome mint.
    const marketAddress = await getMarketPda(id);
    const market = await fetchMarket(client.rpc, marketAddress);
    const outcomeMint =
      market.resolution === MarketResolution.Yes ? market.outcomeYesMint : market.outcomeNoMint;

    // claim fees.
    const ix = await buildClaimFeesIx(accounts, { id, outcomeMint });
    const txSig = await buildAndSendTransaction(client, [ix], {
      feePayer: accounts.briber,
    });
    console.log("claim_fees tx:", txSig);

    // after claiming, available fees must be 0.
    const marketAfter = await fetchMarket(client.rpc, marketAddress);
    expect(marketAfter.availableYesFees).to.equal(0n);
    expect(marketAfter.availableNoFees).to.equal(0n);
  });
});
