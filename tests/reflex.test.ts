import { KeyPairSigner } from "@solana/kit";
import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { buildAddIncentivesIx } from "./instructions/add_incentives";
import { buildAndSendTransaction } from "./utils/tx";
import { buildCreateMarketIxs } from "./instructions/create_market";
import { buildInitializeIx } from "./instructions/initialize";
import { createAccounts } from "./utils/accounts";
import { createClient } from "./utils/client";
import { createMint } from "./utils/mint";
import { expect } from "chai";

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
    const ix = await buildInitializeIx(client);
    const txSig = await buildAndSendTransaction(client, [ix]);
    console.log("initialize tx:", txSig);
    expect(true).to.be.true;
  });

  it("--- create_market ix ---", async () => {
    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";

    const ixs = await buildCreateMarketIxs(client, accounts, {
      id,
      amount: BigInt(10 * LAMPORTS_PER_SOL),
      yesMint,
      noMint,
    });

    const txSig = await buildAndSendTransaction(client, ixs, {
      feePayer: accounts.briber,
      additionalSigners: [client.wallet],
    });
    console.log("create_market tx:", txSig);
    expect(true).to.be.true;
  });

  it("--- add_incentives ix ---", async () => {
    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";

    const ix = await buildAddIncentivesIx(accounts, {
      id,
      amount: BigInt(5 * LAMPORTS_PER_SOL),
    });

    const txSig = await buildAndSendTransaction(client, [ix], {
      feePayer: accounts.briber,
    });
    console.log("add_incentives tx:", txSig);
    expect(true).to.be.true;
  });
});
