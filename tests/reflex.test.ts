import {
  ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
  TOKEN_PROGRAM_ADDRESS,
  findAssociatedTokenPda,
  getCreateAssociatedTokenInstructionAsync,
  getInitializeMintInstruction,
  getMintSize,
  getSyncNativeInstruction,
} from "@solana-program/token";
import {
  AccountRole,
  Instruction,
  KeyPairSigner,
  addSignersToTransactionMessage,
  appendTransactionMessageInstructions,
  assertIsSendableTransaction,
  assertIsTransactionWithBlockhashLifetime,
  createTransactionMessage,
  generateKeyPairSigner,
  getSignatureFromTransaction,
  pipe,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  signTransactionMessageWithSigners,
} from "@solana/kit";
import {
  SYSTEM_PROGRAM_ADDRESS,
  getCreateAccountInstruction,
  getTransferSolInstruction,
} from "@solana-program/system";
import { getConfigPda, getMarketPda, getMarketVaultPda, getTreasuryPda } from "./utils/pda";

import { LAMPORTS_PER_SOL } from "@solana/web3.js";
import { constants } from "./utils/constants";
import { createAccounts } from "./utils/accounts";
import { createClient } from "./utils/client";
import { expect } from "chai";

describe("reflex", () => {
  let client: Awaited<ReturnType<typeof createClient>>;
  let accounts: Awaited<ReturnType<typeof createAccounts>>;
  let yesMint: KeyPairSigner;
  let noMint: KeyPairSigner;

  before(async () => {
    client = await createClient();
    accounts = await createAccounts(client);

    // Prepare inputs.
    const mintSize = getMintSize();
    const [mint1, mint2, mintRent, { value: latestBlockhash }] = await Promise.all([
      generateKeyPairSigner(),
      generateKeyPairSigner(),
      client.rpc.getMinimumBalanceForRentExemption(BigInt(mintSize)).send(),
      client.rpc.getLatestBlockhash().send(),
    ]);

    yesMint = mint1;
    noMint = mint2;

    // Build instructions.
    const createAccountYesIx = getCreateAccountInstruction({
      payer: client.wallet,
      newAccount: yesMint,
      space: mintSize,
      lamports: mintRent,
      programAddress: TOKEN_PROGRAM_ADDRESS,
    });
    const initializeMintYesIx = getInitializeMintInstruction({
      mint: yesMint.address,
      decimals: 6,
      mintAuthority: client.wallet.address,
      freezeAuthority: client.wallet.address,
    });

    const createAccountNoIx = getCreateAccountInstruction({
      payer: client.wallet,
      newAccount: noMint,
      space: mintSize,
      lamports: mintRent,
      programAddress: TOKEN_PROGRAM_ADDRESS,
    });
    const initializeMintNoIx = getInitializeMintInstruction({
      mint: noMint.address,
      decimals: 6,
      mintAuthority: client.wallet.address,
      freezeAuthority: client.wallet.address,
    });

    const ixs = [createAccountYesIx, initializeMintYesIx, createAccountNoIx, initializeMintNoIx];

    const transactionMessage = pipe(
      createTransactionMessage({ version: 0 }),
      (tx) => setTransactionMessageFeePayerSigner(client.wallet, tx),
      (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      (tx) => appendTransactionMessageInstructions(ixs, tx),
    );

    // Compile the transaction message and sign it.
    const transaction = await signTransactionMessageWithSigners(transactionMessage);
    assertIsSendableTransaction(transaction);
    assertIsTransactionWithBlockhashLifetime(transaction);

    // Send the transaction and wait for confirmation.
    await client.sendAndConfirmTransaction(transaction, { commitment: "confirmed" });
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
      expect.fail("Transaction failed: " + err);
    }
    expect(true).to.be.true;
  });

  it("--- create_market ix ---", async () => {
    const { value: latestBlockhash } = await client.rpc.getLatestBlockhash().send();

    // [discriminator, amount, briber_fee_bps]
    const fistData = Buffer.alloc(9);
    fistData.writeUInt8(1, 0);
    fistData.writeBigUInt64LE(BigInt(10 * LAMPORTS_PER_SOL), 1);

    const id = "KXNCAAFGAME-26JAN19MIAIND-IND";
    const secondData = Buffer.from(id, "utf8");

    const ixData = Buffer.concat([fistData, secondData]);

    /*
         x   authority,
         x   config,
         x   briber,
         x   market, 
         x   incentive_mint,
         x   briber_ata,
         x   market_incentive_vault,
         x   treasury,
         x   outcome_yes_mint, // on-chain validation
         x   outcome_no_mint,  // on-chain validation
         x   market_yes_vault,
         x   market_no_vault,
         x   token_program,
         x   _associated_token_program,
         x   _system_program,
    */

    const marketPda = await getMarketPda(id);

    const [briberAta] = await findAssociatedTokenPda({
      mint: constants.WSOL_MINT,
      owner: accounts.briber.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    const createBriberAtaIx = await getCreateAssociatedTokenInstructionAsync({
      payer: accounts.briber,
      ata: briberAta,
      owner: accounts.briber.address,
      mint: constants.WSOL_MINT,
    });

    const transferSolToAtaIx = getTransferSolInstruction({
      source: accounts.briber,
      destination: briberAta,
      amount: BigInt(100 * LAMPORTS_PER_SOL),
    });

    const syncNativeIx = getSyncNativeInstruction({
      account: briberAta,
    });

    const ix: Instruction = {
      programAddress: constants.PROGRAM_ID,
      accounts: [
        {
          address: client.wallet.address,
          role: AccountRole.READONLY_SIGNER,
        },
        {
          address: await getConfigPda(),
          role: AccountRole.READONLY,
        },
        {
          address: accounts.briber.address,
          role: AccountRole.WRITABLE_SIGNER,
        },
        {
          address: marketPda,
          role: AccountRole.WRITABLE,
        },
        {
          address: constants.WSOL_MINT,
          role: AccountRole.READONLY,
        },
        {
          address: briberAta,
          role: AccountRole.WRITABLE,
        },
        {
          address: await getMarketVaultPda(marketPda, constants.WSOL_MINT),
          role: AccountRole.WRITABLE,
        },
        {
          address: await getTreasuryPda(constants.WSOL_MINT),
          role: AccountRole.WRITABLE,
        },
        {
          address: yesMint.address,
          role: AccountRole.READONLY,
        },
        {
          address: noMint.address,
          role: AccountRole.READONLY,
        },
        {
          address: await getMarketVaultPda(marketPda, yesMint.address),
          role: AccountRole.WRITABLE,
        },
        {
          address: await getMarketVaultPda(marketPda, noMint.address),
          role: AccountRole.WRITABLE,
        },
        {
          address: TOKEN_PROGRAM_ADDRESS,
          role: AccountRole.READONLY,
        },
        {
          address: ASSOCIATED_TOKEN_PROGRAM_ADDRESS,
          role: AccountRole.READONLY,
        },
        {
          address: SYSTEM_PROGRAM_ADDRESS,
          role: AccountRole.READONLY,
        },
      ],
      data: ixData,
    };

    console.log("briber:", accounts.briber.address);
    console.log("wallet:", client.wallet.address);

    const ixs = [createBriberAtaIx, transferSolToAtaIx, syncNativeIx, ix];

    const transactionMessage = pipe(
      createTransactionMessage({ version: 0 }),
      (tx) => setTransactionMessageFeePayerSigner(accounts.briber, tx),
      (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
      (tx) => appendTransactionMessageInstructions(ixs, tx),
      (tx) => addSignersToTransactionMessage([client.wallet, accounts.briber], tx),
    );

    // Compile the transaction message and sign it.
    const transaction = await signTransactionMessageWithSigners(transactionMessage);
    assertIsSendableTransaction(transaction);
    assertIsTransactionWithBlockhashLifetime(transaction);

    const txSig = getSignatureFromTransaction(transaction);

    try {
      await client.sendAndConfirmTransaction(transaction, { commitment: "confirmed" });
      console.log("create_market tx:", txSig);
    } catch (err) {
      console.log("error:", err);
      expect.fail("Transaction failed: " + err);
    }
    expect(true).to.be.true;
  });
});
