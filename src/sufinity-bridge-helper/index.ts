import { Secp256k1PublicKey } from "@mysten/sui.js/keypairs/secp256k1";

export const getSuiAddress = async () => {
  let publicKey = new Secp256k1PublicKey(
    "AkdNhFHqiMAre5GIQX5ZzXmKuQInrfc0juNJSEPNaQKh"
  );

  let suiAddress = publicKey.toSuiAddress();
  console.log("Sui address", suiAddress);
};

getSuiAddress();
