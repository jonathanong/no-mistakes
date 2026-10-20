import sql from "sql-template-strings";

// Tagged fragment constant. A same-file append already copies this; only the
// imported constant form used to stay unanalyzable.
export const FRAG = sql` AND account_id = $1`;
