import { KeyPairSigner, generateKeyPairSigner } from "@solana/kit";
import {
  TOKEN_PROGRAM_ADDRESS,
  getInitializeMintInstruction,
  getMintSize,
} from "@solana-program/token";

import { Client } from "./client";
import { buildAndSendTransaction } from "./tx";
import { getCreateAccountInstruction } from "@solana-program/system";

/**
 * Creates and initializes a new SPL token mint.
 * Returns the mint keypair signer.
 */
export async function createMint(client: Client, decimals: number = 6): Promise<KeyPairSigner> {
  const mintKp = await generateKeyPairSigner();
  const mintSize = getMintSize();
  const mintRent = await client.rpc.getMinimumBalanceForRentExemption(BigInt(mintSize)).send();

  const createAccountIx = getCreateAccountInstruction({
    payer: client.wallet,
    newAccount: mintKp,
    space: mintSize,
    lamports: mintRent,
    programAddress: TOKEN_PROGRAM_ADDRESS,
  });

  const initializeMintIx = getInitializeMintInstruction({
    mint: mintKp.address,
    decimals,
    mintAuthority: client.wallet.address,
    freezeAuthority: client.wallet.address,
  });

  await buildAndSendTransaction(client, [createAccountIx, initializeMintIx]);
  return mintKp;
}
