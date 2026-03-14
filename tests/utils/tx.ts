import {
  Instruction,
  MessageSigner,
  TransactionSigner,
  addSignersToTransactionMessage,
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

import { Client } from "./client";

export type SendOptions = {
  feePayer?: TransactionSigner & MessageSigner;
  additionalSigners?: (TransactionSigner & MessageSigner)[];
};

/**
 * Builds, signs and sends a transaction with the given instructions.
 * Returns the transaction signature.
 */
export async function buildAndSendTransaction(
  client: Client,
  instructions: Instruction[],
  { feePayer, additionalSigners = [] }: SendOptions = {},
): Promise<string> {
  const { value: latestBlockhash } = await client.rpc.getLatestBlockhash().send();
  const payer = feePayer ?? client.wallet;

  const txMessage = pipe(
    createTransactionMessage({ version: 0 }),
    (tx) => setTransactionMessageFeePayerSigner(payer, tx),
    (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    (tx) => appendTransactionMessageInstructions(instructions, tx),
    (tx) => addSignersToTransactionMessage(additionalSigners, tx),
  );

  const transaction = await signTransactionMessageWithSigners(txMessage);
  assertIsSendableTransaction(transaction);
  assertIsTransactionWithBlockhashLifetime(transaction);

  const txSig = getSignatureFromTransaction(transaction);
  await client.sendAndConfirmTransaction(transaction, { commitment: "confirmed" });
  return txSig;
}
