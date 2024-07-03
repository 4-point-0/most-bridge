"use client";

import * as React from "react";
import {
  ColumnDef,
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  useReactTable,
} from "@tanstack/react-table";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

export type TransactionBurn = {
  date: string;
  amount: number;
  from: string;
  block_index: string;
  tx: string;
};

export const columns: ColumnDef<TransactionBurn>[] = [
  {
    accessorKey: "block_index",
    header: "Block index",
    cell: ({ row }) => (
      <div className="lowercase">{row.getValue("block_index")}</div>
    ),
  },
  {
    accessorKey: "date",
    header: "Date",
    cell: ({ row }) => <div className="lowercase">{row.getValue("date")}</div>,
  },
  {
    accessorKey: "amount",
    header: "Amount",
    cell: ({ row }) => <div>{row.getValue("amount") + " MIST"}</div>,
  },
  {
    accessorKey: "token_sybmol",
    header: "Token symbol",
    cell: ({ row }) => <div>{"ckSUI"}</div>,
  },
  {
    accessorKey: "from",
    header: "From",
    cell: ({ row }) => <div className="lowercase">{row.getValue("from")}</div>,
  },

  {
    accessorKey: "tx",
    header: "Tx on SUI",
    cell: ({ row }) => (
      <div className="lowercase">
        {
          <a target="_blank" href={row.getValue("tx")}>
            {row.getValue("tx")}
          </a>
        }
      </div>
    ),
  },
];

type Props = {
  data: TransactionBurn[];
  isLoading: boolean;
};

export function TransactionBurnTable(props: Props) {
  const table = useReactTable({
    data: props.data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
  });
  const { isLoading } = props;
  return (
    <div className="w-full pt-5 px-10">
      <h3 className="scroll-m-20 text-1xl pb-2 font-semibold tracking-tight">
        Finalized transactions ckSUI -&gt; SUI{" "}
      </h3>
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            {table.getHeaderGroups().map((headerGroup) => (
              <TableRow key={headerGroup.id}>
                {headerGroup.headers.map((header) => {
                  return (
                    <TableHead key={header.id}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                    </TableHead>
                  );
                })}
              </TableRow>
            ))}
          </TableHeader>
          <TableBody>
            {table.getRowModel().rows?.length ? (
              table.getRowModel().rows.map((row) => (
                <TableRow
                  key={row.id}
                  data-state={row.getIsSelected() && "selected"}
                >
                  {row.getVisibleCells().map((cell) => (
                    <TableCell key={cell.id}>
                      {flexRender(
                        cell.column.columnDef.cell,
                        cell.getContext()
                      )}
                    </TableCell>
                  ))}
                </TableRow>
              ))
            ) : (
              <TableRow>
                <TableCell
                  colSpan={columns.length}
                  className="h-24 text-center"
                >
                  {isLoading ? (
                    <div
                      className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-current border-e-transparent align-[-0.125em] text-surface motion-reduce:animate-[spin_1.5s_linear_infinite] dark:text-white"
                      role="status"
                    >
                      <span className="!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]" />
                    </div>
                  ) : (
                    <div>No results.</div>
                  )}
                </TableCell>
              </TableRow>
            )}
          </TableBody>
        </Table>
      </div>
    </div>
  );
}
