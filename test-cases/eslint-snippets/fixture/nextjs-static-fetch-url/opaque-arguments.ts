declare const args: Parameters<typeof fetch>;
declare const method: string;
declare const overrides: RequestInit;

fetch(...args);
fetch.apply(undefined, args);
fetch.call(undefined, ...args);
fetch("/widgets", { method });
fetch("/widgets", { method: "GET", ...overrides });
fetch("/widgets", { ...overrides, method: "GET" });
fetch("/widgets", ...args);
fetch.call(undefined, "/widgets", ...args);
fetch.apply(undefined, ["/widgets", ...args]);
fetch.apply(undefined, ["/widgets", { method: "GET" }]);
fetch("/widgets");
fetch("/widgets", { ...{ cache: "no-store" }, method: "GET" });
fetch("/widgets", { method: "GET", ...{ cache: "no-store" } });
fetch("/widgets", { method: "GET", ...{ method } });

function unrelated(fetch: (...args: unknown[]) => void) {
  fetch.call(undefined, ...args);
  fetch.apply(undefined, args);
}
