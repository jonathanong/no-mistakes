import { transaction } from "@sample/database";
import { formatId } from "@sample/utils";
import { missingTx, opaqueRunTx } from "../db-named";
import { missingDirectTx } from "../db-missing";
import { missingDirectSend } from "../jobs-missing";
import {
  constNsAlias,
  localCycleA,
  mutableSend,
  objHelpers,
  opaqueMemberSend,
  opaqueSend,
  spreadJobs,
} from "../jobs-named";

declare const method: string;
declare const optionalFormat: typeof formatId | undefined;
declare const unknownQueues: Record<string, { add: (job: { receiptId: string }) => Promise<void> }>;
declare class CustomQueue {
  add(job: { receiptId: string }): Promise<void>;
}

export function mapOpaqueLocalExports(rows: { id: string }[]) {
  return transaction("write", async () => {
    rows.map(opaqueMemberSend);
    rows.map(spreadJobs.enqueue);
    rows.map(constNsAlias as unknown as (row: { id: string }) => unknown);
    rows.map(mutableSend);
    rows.map(localCycleA);
    rows.map((objHelpers as unknown as { missing: (row: { id: string }) => unknown }).missing);
    await opaqueSend({ id: "direct-opaque-publish" });
    await missingDirectSend({ id: "missing-direct-publish" });
    const dynamicQueue = unknownQueues[method];
    await dynamicQueue.add({ receiptId: "aliased-dynamic-receiver" });
    let mutableSet = new Set<string>();
    mutableSet = unknownQueues[method] as unknown as Set<string>;
    mutableSet.add("reassigned-receiver");
    const customQueue = new CustomQueue();
    await customQueue.add({ receiptId: "custom-ctor" });
    const r1 = new Set<string>();
    const r2 = r1;
    const r3 = r2;
    const r4 = r3;
    const r5 = r4;
    const r6 = r5;
    r6.add("too-deep-receiver-alias");
    await (optionalFormat ?? formatId)({ id: "ambiguous-unknown-other" });
  });
}

export async function runOpaqueAndMissingTransactions() {
  await opaqueRunTx("write", () => {});
  await missingTx("write", () => {});
  await missingDirectTx("write", () => {});
  await (opaqueRunTx as unknown as { run: typeof transaction }).run("write", () => {});
}
