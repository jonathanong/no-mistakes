declare const dyn: string;
fetch("/api", { method: dyn });
fetch("/api", { method: dyn } as RequestInit);
fetch("/api", { method: dyn } satisfies RequestInit);
fetch("/api", { ["method" as const]: dyn });
fetch("/api", { method: "POST" as const });
