"use client";
import fetch from "isomorphic-fetch";
import { Actor, HttpAgent } from "@dfinity/agent";
// import { idlFactory as ledgerIdl } from "./icrc1_ledger_canister.did";
import { minterFactory as minterIdl } from "./minter.did";
import { useEffect, useState } from "react";
import moment from "moment";
import {
  TransactionMint,
  TransactionMintTable,
} from "./transaction-mint-table";
import { useTheme } from "next-themes";
import { TransactionBurnTable } from "./transaction-burn-table";

const host =
  "http://127.0.0.1:4943/?canisterId=bnz7o-iuaaa-aaaaa-qaaaa-cai&id=bd3sg-teaaa-aaaaa-qaaba-cai";

const canisterId = "bd3sg-teaaa-aaaaa-qaaba-cai";

/**
 * @type {RequestInit}
 */
const fetchOptions = {
  headers: {
    "Content-Type": "application/json",
  },
};

export default function Home() {
  const [mintedTransactions, setMintedTransactions] = useState<
    TransactionMint[]
  >([]);
  const [finalizedTransactions, setFinalizedTransactions] = useState([]);
  const [isLoadingMinted, setIsLoadingMinted] = useState(true);
  const [isLoadingFinalized, setIsLoadingFinalized] = useState(true);

  const { setTheme } = useTheme();

  const callGetTransactions = async () => {
    try {
      const agent = new HttpAgent({
        fetch,
        host: process.env.NEXT_PUBLIC_HOST!,
        fetchOptions,
      });

      const actor = Actor.createActor(minterIdl, {
        agent,
        canisterId: process.env.NEXT_PUBLIC_CANISTER_ID!,
      });

      setIsLoadingMinted(true);
      await agent.fetchRootKey();
      const get_minted_transactions = await actor.get_minted_transactions();
      const minted_trasactions = (get_minted_transactions as any).map(
        (transaction: any) => {
          let parsed = JSON.parse(transaction);

          return {
            block_index: parsed.block_index,
            date: moment(
              new Date(Number(BigInt(parsed.date) / BigInt(1000000)))
            ).format("DD-MM-YYYY hh:mm:ss"),
            amount: parsed.amount,
            from: parsed.from,
            to: parsed.to,
          };
        }
      );
      setMintedTransactions(minted_trasactions);
      setIsLoadingMinted(false);
      setIsLoadingFinalized(true);

      const finalized = await actor.get_finalized_transactions();
      const finalized_trasactions = (finalized as any).map(
        (transaction: any) => {
          let parsed = JSON.parse(transaction);

          return {
            block_index: parsed.block_index,
            date: moment(
              new Date(Number(BigInt(parsed.date) / BigInt(1000000)))
            ).format("DD-MM-YYYY hh:mm:ss"),
            amount: parsed.amount,
            from: parsed.from,
            tx: parsed.tx,
          };
        }
      );

      setFinalizedTransactions(finalized_trasactions);
      setIsLoadingFinalized(false);
    } catch (error) {
      console.error("Failed to call get_transactions:", error);
    }
  };

  useEffect(() => {
    setTheme("dark");
    callGetTransactions();
  }, []);

  return (
    <main className="relative flex h-screen flex-grow flex-col items-center pt-10">
      <h2 className="scroll-m-20 text-left pb-5 px-10 text-3xl font-semibold tracking-tight first:mt-0">
        Minter explorer
      </h2>
      <TransactionMintTable
        isLoading={isLoadingMinted}
        data={mintedTransactions}
      />
      <TransactionBurnTable
        isLoading={isLoadingFinalized}
        data={finalizedTransactions}
      />
    </main>
  );
}
