import { read } from '@bench/db';
// The legacy String tag contract requires a local function with this shape.
function String(strings: TemplateStringsArray, ...values: unknown[]) { return strings.join('?'); }
read(String`SELECT 5`);
