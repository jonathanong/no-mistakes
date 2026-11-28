declare const dyn: string;
fetch("/api/users");
fetch("/api/users" as const);
fetch("/api/users" satisfies string);
globalThis["fetch" as const](dyn);
