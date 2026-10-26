import { read } from "@bench/db";
import { sql } from "@bench/tags";
const safe = sql`/* safe */ SELECT 1`;
read(safe); // low: true
read(safe); // low: true
const changed = sql`/* changed */ SELECT 1`;
changed.text = "SELECT unsafe";
read(changed); // low: false
const computed = sql`/* computed */ SELECT 1`;
computed["text"] = "SELECT unsafe";
read(computed); // low: false
const prepended = sql`/* prepend */ SELECT 1`;
prepended.prepend("SELECT unsafe");
read(prepended); // low: false
const escaped = sql`/* escaped */ SELECT 1`;
escapeToHelper(escaped);
read(escaped); // low: false
const aliased = sql`/* aliased */ SELECT 1`;
const alias = aliased;
read(aliased); // low: false
const returned = sql`/* returned */ SELECT 1`;
function leak() { return returned; }
read(returned); // low: false
const stored = sql`/* stored */ SELECT 1`;
const holder = { stored };
read(stored); // low: false
const captured = sql`/* captured */ SELECT 1`;
function mutate() { captured.text = "SELECT unsafe"; }
read(captured); // low: false
export const exported = sql`/* exported */ SELECT 1`;
read(exported); // low: false
const separatelyExported = sql`/* exported separately */ SELECT 1`;
export { separatelyExported };
read(separatelyExported); // low: false
const literal = "/* literal */ SELECT 1";
escapeToHelper(literal);
read(literal); // low: true
const capturedSafe = sql`/* captured safe */ SELECT 1`;
function execute() { read(capturedSafe); } // low: true

// A tagged left operand keeps tag-constant ownership checks after concatenation.
const plusSafe = sql`/* safe plus */ SELECT 1` + tail;
read(plusSafe); // low: true
const plusChanged = sql`/* changed plus */ SELECT 1` + tail;
plusChanged.text = "SELECT unsafe";
read(plusChanged); // low: false
const plusEscaped = sql`/* escaped plus */ SELECT 1` + tail;
escapeToHelper(plusEscaped);
read(plusEscaped); // low: false
