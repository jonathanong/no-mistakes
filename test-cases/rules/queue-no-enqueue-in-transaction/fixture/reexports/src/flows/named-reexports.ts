import { beginTransaction, beginTxLocal, localBeginTransaction } from "../db-named";
import * as jobsBarrelNs from "../jobs-named";
import {
  enqueue,
  fmt,
  jobsNs,
  localJobsNs,
  localSend,
  send,
  sendLocal,
  utilsDefault,
} from "../jobs-named";
import { dispatch } from "../jobs-hop";

export function publishThroughReexports(rows: { id: string }[]) {
  return beginTransaction("write", async () => {
    await enqueue({ receiptId: "direct-named-reexport" });
    await send({ receiptId: "renamed-direct-reexport" });
    rows.map(enqueue);
    rows.map(send);
    rows.map(dispatch);
    rows.map(localSend);
    rows.map(jobsNs.enqueue);
    rows.map(localJobsNs.enqueue);
    rows.map(jobsBarrelNs.send);
    rows.map(fmt);
    rows.map(utilsDefault.format);
  });
}

export function publishInsideLocalReexportedBegin(rows: { id: string }[]) {
  return beginTxLocal(async () => {
    await sendLocal({ receiptId: "local-reexported-direct" });
    rows.map(sendLocal);
  });
}

export function publishInsideLocalImportExportBegin(rows: { id: string }[]) {
  return localBeginTransaction("write", async () => {
    rows.map(send);
  });
}
