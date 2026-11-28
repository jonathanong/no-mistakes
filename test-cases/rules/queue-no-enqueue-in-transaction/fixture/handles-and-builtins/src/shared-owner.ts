import { withTx } from "./tx";
import { enqueue } from "@sample/jobs";

declare const flag: boolean;

export function sharedHelperInNestedTransaction(rows: { id: string }[]) {
  return withTx(async (outer) => {
    const work = () => {
      void outer(`SELECT 1`);
      return enqueue({ id: "shared" });
    };
    rows.map(work);
    return withTx(async (inner) => {
      rows.map(work);
      return inner(`SELECT 2`);
    });
  });
}

export function conditionalReplacer(data: { id: string }) {
  return withTx(async () => JSON.stringify(data, flag ? enqueue : null));
}
