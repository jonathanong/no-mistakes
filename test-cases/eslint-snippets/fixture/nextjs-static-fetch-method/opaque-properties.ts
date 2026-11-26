declare const overrides: RequestInit;
declare const key: string;

fetch("/widgets", { ...overrides });
fetch("/widgets", { method: "GET", [key]: overrides });
fetch("/widgets", { [key]: overrides, method: "GET" });
