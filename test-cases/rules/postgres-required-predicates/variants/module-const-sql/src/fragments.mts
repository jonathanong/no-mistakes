import sql from "sql-template-strings";

// The binding is const, but an importer can still mutate this builder object.
// Cross-module recovery must abstain rather than splice its initial snapshot.
export const FRAG = sql` AND account_id = $1`;
