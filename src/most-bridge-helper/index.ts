import { Secp256k1PublicKey } from "@mysten/sui.js/keypairs/secp256k1";

export const getSuiAddress = async (address: string) =>
  console.log(new Secp256k1PublicKey(address).toSuiAddress());

getSuiAddress(process.env.ADDRESS!);
