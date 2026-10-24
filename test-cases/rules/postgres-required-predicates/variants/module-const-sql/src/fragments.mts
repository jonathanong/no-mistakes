import sql from "sql-template-strings";

// Const fixes this binding, not the statement object. Imported append must stay
// unknown because another importer can mutate the same object.
export const FRAG = sql` AND account_id = $1`;
