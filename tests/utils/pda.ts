import { Address, getAddressEncoder, getProgramDerivedAddress } from "@solana/kit";

import { constants } from "./constants";

const getConfigPda = async () => {
  const [config, _] = await getProgramDerivedAddress({
    programAddress: constants.PROGRAM_ID,
    seeds: [constants.CONFIG_SEED],
  });

  return config;
};

const getTreasuryPda = async (mint: Address) => {
  const [treasury, _] = await getProgramDerivedAddress({
    programAddress: constants.PROGRAM_ID,
    seeds: [constants.TREASURY_SEED, getAddressEncoder().encode(mint)],
  });

  return treasury;
};

const getMarketPda = async (id: string) => {
  const [market, _] = await getProgramDerivedAddress({
    programAddress: constants.PROGRAM_ID,
    seeds: [constants.MARKET_SEED, Buffer.from(id, "utf-8")],
  });

  return market;
};

const getMarketVaultPda = async (market: Address, mint: Address) => {
  const [vault, _] = await getProgramDerivedAddress({
    programAddress: constants.PROGRAM_ID,
    seeds: [
      constants.MARKET_SEED,
      getAddressEncoder().encode(market),
      getAddressEncoder().encode(mint),
    ],
  });

  return vault;
};

export { getConfigPda, getTreasuryPda, getMarketPda, getMarketVaultPda };
