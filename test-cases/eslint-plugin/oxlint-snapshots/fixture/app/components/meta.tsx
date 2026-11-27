import { describe, test } from "vitest";

const values = [3, 1, 2];
const optionalValues = [3, 1, 2];
const [destructuredClient] = [repo];
let reassignedValues = [3, 1, 2];
reassignedValues = query;
await task;
await values.map((value) => value);
await sort();
await optionalValues?.sort();
await optionalValues["sort"]();
await optionalValues[method]();
await getValues().sort();
await destructuredClient.sort();
await reassignedValues.sort();
void value;
void (value as unknown);
delete (value as { x: string }).x;
const maybe = value as { x: string };
delete maybe.x;
export { metadata };
// Oxlint rejects delete of an unqualified name and then skips the plugin. Those two lines are void expressions so the export stays on line 22.
