// One `export *` source declares the namespace `Clash` and the other exports a
// value of that name, so which one `Clash` means is ambiguous.
export * from "./clash-namespace";
export * from "./clash-value";
