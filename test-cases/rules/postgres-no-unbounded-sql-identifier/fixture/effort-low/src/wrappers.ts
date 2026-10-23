import { withTransaction } from "@example/db";
declare function log(): void;
// Legacy max gives this configured-module import implicit executor semantics.
// Low deliberately avoids callback inference even when this callback is SQL-free.
withTransaction(() => log()); // unsupported: legacy transaction wrapper
