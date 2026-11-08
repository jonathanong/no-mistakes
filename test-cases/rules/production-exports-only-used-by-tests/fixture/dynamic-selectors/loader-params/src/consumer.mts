const registry = { x: (unused) => import("./worker.mjs") };
(await registry.x()).default();
