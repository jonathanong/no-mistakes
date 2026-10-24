import { withTransaction } from "@example/db";

// For the direct BEGIN, namespace COMMIT and direct ROLLBACK probes, migrate
// the complete surrounding lifecycle into one owner-helper operation.
await withTransaction(async (tx) => {
  await tx.query("/* direct-operation */ UPDATE users SET active = true");
});

// The isolated .call/.apply probes have a null receiver and only pure SQL reads.
// Their complete operation also uses the helper's executor; no manual command remains.
await withTransaction(async (tx) => {
  await tx.query("/* forwarded-operation */ UPDATE users SET active = true");
});

// Move the lifecycle around a transparent executor adapter into the owner helper.
// Other adapter uses can remain; transaction ownership moves to this callback.
await withTransaction(async (tx) => {
  await tx.query("/* adapter-operation */ UPDATE users SET active = true");
});
