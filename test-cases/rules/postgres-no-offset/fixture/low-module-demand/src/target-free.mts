declare function unrelated(value: unknown): void;
function hidden() { return 'SELECT 1 OFFSET 3'; }
const local = { sql: hidden() };
unrelated(local);
