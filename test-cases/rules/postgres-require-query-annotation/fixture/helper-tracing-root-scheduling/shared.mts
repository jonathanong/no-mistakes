// Each independent root initializes this mutable module and explores both arms.
let sql = "SELECT 80";
if (globalThis.choice) { sql = "SELECT 81"; } else { sql = "SELECT 82"; }
export function statement() { return sql; }
