import type { enqueueBatch, computeDelay } from "@app/jobs";

type Dependencies = {
  enqueueBatch?: typeof enqueueBatch;
  computeDelay?: typeof computeDelay;
};

export async function drainGroups(rows: unknown[], dependencies: Required<Dependencies>) {
  const delay = await dependencies.computeDelay("group");
  await dependencies.enqueueBatch(rows, { delay });
}

export async function drainDropped(rows: unknown[], dependencies: Required<Dependencies>) {
  // Awaiting the wrapper does not adopt a callback promise dropped inside it.
  const delay = await dependencies.computeDelay("group");
  dependencies.enqueueBatch(rows, { delay });
}
