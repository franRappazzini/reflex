import {
  MessageSigner,
  TransactionSigner,
  airdropFactory,
  generateKeyPairSigner,
  lamports,
} from "@solana/kit";

import { Client } from "./client";

export type Accounts = {
  briber: TransactionSigner & MessageSigner;
  farmer: TransactionSigner & MessageSigner;
};

let accounts: Accounts | undefined;
export async function createAccounts(client: Client): Promise<Accounts> {
  if (!accounts) {
    const airdrop = airdropFactory({ rpc: client.rpc, rpcSubscriptions: client.rpcSubscriptions });

    // Create a briber with lamports.
    const briber = await generateKeyPairSigner();
    await airdrop({
      recipientAddress: briber.address,
      lamports: lamports(1_000_000_000_000n),
      commitment: "confirmed",
    });

    // Create a farmer with lamports.
    const farmer = await generateKeyPairSigner();
    await airdrop({
      recipientAddress: farmer.address,
      lamports: lamports(1_000_000_000_000n),
      commitment: "confirmed",
    });

    // Store the accounts.
    accounts = { briber, farmer };
  }
  return accounts;
}
