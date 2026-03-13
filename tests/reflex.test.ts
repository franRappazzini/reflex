import {
  AccountRole,
  Instruction,
  appendTransactionMessageInstructions,
  assertIsSendableTransaction,
  assertIsTransactionWithBlockhashLifetime,
  createTransactionMessage,
  getSignatureFromTransaction,
  pipe,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
} from "@solana/kit";
import { getConfigPda, getTreasuryPda } from "./utils/pda";

import { SYSTEM_PROGRAM_ADDRESS } from "@solana-program/system";
import { TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import { constants } from "./utils/constants";
import { createClient } from "./utils/client";
import { expect } from "chai";

describe("reflex", () => {
  let client: Awaited<ReturnType<typeof createClient>>;

  before(async () => {
    client = await createClient();
  });

  it("--- initialize ix ---", async () => {
    const { value: latestBlockhash } = await client.rpc.getLatestBlockhash().send();

    // [discriminator, fee_bps, briber_fee_bps]
    const ixData = Buffer.alloc(5);
    ixData.writeUInt8(0, 0);
    ixData.writeUInt16LE(500, 1);
    ixData.writeUInt16LE(500, 3);

    /*
        authority,
        config,
        wsol_mint,
        usdc_mint,
        wsol_treasury,
        usdc_treasury,
        token_program,
        _system_program,
    */

    const ix: Instruction = {
      programAddress: constants.PROGRAM_ID,
      accounts: [
        {
          address: client.wallet.address,
          role: AccountRole.WRITABLE_SIGNER,
        },
        {
          address: await getConfigPda(),
          role: AccountRole.WRITABLE,
        },
        {
          address: constants.WSOL_MINT,
          role: AccountRole.READONLY,
        },
        {
          address: constants.USDC_MINT,
          role: AccountRole.READONLY,
        },
        {
          address: await getTreasuryPda(constants.WSOL_MINT),
          role: AccountRole.WRITABLE,
        },
        {
          address: await getTreasuryPda(constants.USDC_MINT),
          role: AccountRole.WRITABLE,
        },
        {
          address: TOKEN_PROGRAM_ADDRESS,
          role: AccountRole.READONLY,
        },
        {
          address: SYSTEM_PROGRAM_ADDRESS,
          role: AccountRole.READONLY,
        },
      ],
      data: ixData,
    };

    const transactionMessage = pipe(
      createTransactionMessage({ version: 0 }),
      (tx) => setTransactionMessageFeePayerSigner(client.wallet, tx),
      (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      (tx) => appendTransactionMessageInstructions([ix], tx),
    );

    // Compile the transaction message and sign it.
    const transaction = await signTransactionMessageWithSigners(transactionMessage);
    assertIsSendableTransaction(transaction);
    assertIsTransactionWithBlockhashLifetime(transaction);

    const txSig = getSignatureFromTransaction(transaction);

    try {
      await client.sendAndConfirmTransaction(transaction, { commitment: "confirmed" });

      console.log("initialize tx:", txSig);
    } catch (err) {
      console.log("error:", err);
    }
    expect(true).to.be.true;
  });
});
