import {
  MessageSigner,
  Rpc,
  RpcSubscriptions,
  SolanaRpcApi,
  SolanaRpcSubscriptionsApi,
  TransactionSigner,
  airdropFactory,
  createSolanaRpc,
  createSolanaRpcSubscriptions,
  generateKeyPairSigner,
  lamports,
  sendAndConfirmTransactionFactory,
} from "@solana/kit";

export type Client = {
  rpc: Rpc<SolanaRpcApi>;
  rpcSubscriptions: RpcSubscriptions<SolanaRpcSubscriptionsApi>;
  sendAndConfirmTransaction: ReturnType<typeof sendAndConfirmTransactionFactory>;
  wallet: TransactionSigner & MessageSigner;
};

let client: Client | undefined;
export async function createClient(): Promise<Client> {
  if (!client) {
    // Create RPC objects and airdrop function.
    const rpc = createSolanaRpc("http://127.0.0.1:8899");
    const rpcSubscriptions = createSolanaRpcSubscriptions("ws://127.0.0.1:8900");
    const airdrop = airdropFactory({ rpc, rpcSubscriptions });

    // Create a wallet with lamports.
    const wallet = await generateKeyPairSigner();
    await airdrop({
      recipientAddress: wallet.address,
      lamports: lamports(1_000_000_000_000n),
      commitment: "confirmed",
    });

    // Create a function to send and confirm transactions.
    const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({
      rpc,
      rpcSubscriptions,
    });

    // Store the client.
    client = { rpc, rpcSubscriptions, wallet, sendAndConfirmTransaction };
  }
  return client;
}
