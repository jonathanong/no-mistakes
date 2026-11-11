import("./bare");
await import("./awaited");
void import("./void");
const unused = await import("./unused");
const typed = await import("./typed");
type Typed = typeof typed;

export const exposed = await import("./exported");
declare function consume(value: unknown): void;
const escaped = await import("./escaped");
consume(escaped);
const promise = import("./promise");
void promise;
const computed = await import("./computed");
declare const key: string;
void computed[key];
