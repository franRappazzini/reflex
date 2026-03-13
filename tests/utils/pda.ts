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

export { getConfigPda, getTreasuryPda };
