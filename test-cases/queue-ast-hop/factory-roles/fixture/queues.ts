import { BACKGROUND_QUEUE_NAME } from "./config";
import { createQueue } from "./lib/factory";

export const background = createQueue(BACKGROUND_QUEUE_NAME, {
  priorityKind: "ranked",
});
export const other = createQueue("other");
export const unattended = createQueue("unattended");
export const adminImports = background;
// These shapes stay: each one is a branch the role resolver has to ignore or
// follow. A member callee and a missing name are not queue definitions, a
// non-identifier name is not an imported constant, and the re-export is only
// a queue name after the imported constant resolves.
const holder = { createQueue };
const memberCall = holder.createQueue("skipped");
const missingArg = createQueue();
const notIdentifier = createQueue(1);
export const fromImport = createQueue(BACKGROUND_QUEUE_NAME);
export { other as published, fromImport as reexportedQueue };
const copied = other;
const importedAlias = BACKGROUND_QUEUE_NAME;
void memberCall;
void missingArg;
void notIdentifier;
void copied;
void importedAlias;
