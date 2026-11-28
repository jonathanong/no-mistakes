import { createDb, transaction } from "@sample/database";
import { enqueue } from "@sample/jobs";
import { getTransaction } from "../db-factory";
import { transaction as starTransaction } from "../db-star";
import { enqueue as starEnqueue } from "../jobs-star";

declare const method: string;
declare const mode: [string];
declare const callbacks: [() => Promise<void>];
declare const optionalTx: typeof transaction | undefined;
declare const optionalEnqueue: typeof enqueue | undefined;
declare const unknownJobs: Record<string, typeof enqueue>;

// Unresolved and ambiguous targets inside a transaction callback.
await transaction("write", async () => {
  await unknownJobs[method]({ receiptId: "dynamic" });
  await starEnqueue({ receiptId: "star-reexport-publish" });
  await (optionalEnqueue ?? enqueue)({ receiptId: "ambiguous-publish" });
});

// Ambiguous transaction target.
await (optionalTx ?? transaction)("write", () => enqueue({ receiptId: "nullish-tx" }));

// Unresolved transaction targets (wildcard re-export and opaque factory member).
await starTransaction("write", () => enqueue({ receiptId: "star-reexport-tx" }));
const dbClient = createDb();
await dbClient.transaction("write", () => enqueue({ receiptId: "factory-member-tx" }));

// Bound transaction call.
const boundTransaction = transaction.bind(null, "write");
await boundTransaction(
  () => "callback-ran",
  () => enqueue({ receiptId: "bound-tx" }),
);

// Spread arguments at or before the callback slot.
await transaction(...mode, () => enqueue({ receiptId: "spread-before" }));
await transaction("write", ...callbacks);

// Non-inline and generator callbacks.
const publish = () => enqueue({ receiptId: "non-inline" });
await transaction("write", publish);
await transaction("write", function* () {
  yield enqueue({ receiptId: "generator" });
});

// Tagged template transaction call.
await transaction`write ${() => enqueue({ receiptId: "tagged" })}`;

// Factory-returned transaction target.
const factoryTransaction = getTransaction();
await factoryTransaction("write", () => enqueue({ receiptId: "factory-tx" }));

// Tagged template on a wildcard-reexported transaction import.
await starTransaction`write ${() => enqueue({ receiptId: "star-reexport-tagged-tx" })}`;
