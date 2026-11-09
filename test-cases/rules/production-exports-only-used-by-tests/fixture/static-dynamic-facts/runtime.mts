let mutable = "./target.mjs";
await import(mutable);
const rewritten = "./target.mjs";
rewritten = "./other.mjs";
await import(rewritten);
const { nested } = { nested: "./target.mjs" };
await import(nested);
const object = new URL("./target.mjs", import.meta.url);
object.pathname = "/other.mjs";
await import(object);
await import(object.href);
await import(unknownTarget);
await import(new URL("https://example.invalid/target.mjs", import.meta.url));
await import(new URL("./target.mjs"));
await import(new URL(mutable, import.meta.url));
await import(new URL("./target.mjs", otherBase));
await import(new URL("./target.mjs", import.meta.other));
await import(new URL("./target.mjs", other.url));
await import(new URL("./target%20name.mjs", import.meta.url).pathname);
await import(new URL("./target.mjs?query", import.meta.url).pathname);
await import(new URL("./target.mjs#fragment", import.meta.url).pathname);
await import(String(mutable));
await import(String(new URL("./target.mjs", import.meta.url), 1));
await import(String?.(new URL("./target.mjs", import.meta.url)));
await import(new URL("./target.mjs", import.meta.url)?.href);
await import(new URL("./target.mjs", import.meta.url).other);
await import(new URL("./target%2fencoded.mjs", import.meta.url));
function shadowUrl(URL) { return import(new URL("./target.mjs", import.meta.url)); }
function shadowString(String) { return import(String(new URL("./target.mjs", import.meta.url))); }
function memberGlobal(globals) { return import(new globals.URL("./target.mjs", import.meta.url)); }
function nonVariable(target) { return import(target); }
await import(beforeDeclaration);
const beforeDeclaration = "./target.mjs";
const cycleA = cycleB;
const cycleB = cycleA;
await import(cycleA);
const fileUrl = "file:///target.mjs";
await import(fileUrl);
const dataUrl = "data:text/javascript,export const value = 1";
await import(dataUrl);
