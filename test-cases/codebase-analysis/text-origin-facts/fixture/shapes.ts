declare const extra: Record<string, unknown>;
declare const key: string;
declare const optional: (() => unknown) | undefined;

const plain = { safe: "ordinary" };
const spread = { safe: "ordinary", ...extra };
const computed = { safe: "ordinary", [key]: "replacement" };
const bigint = { safe: "ordinary", 1n: "replacement" };
const accessor = { safe: "ordinary", get other() { return "replacement"; } };
const method = { safe: "ordinary", other() { return "replacement"; } };
const mismatchedParts = `prefix_${"ordinary"}`;

function patternDefault({ value = optional?.() }, unused) {}
