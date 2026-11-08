const registry = { x: () => import("./worker.mjs") };
registry.x = other;
(await registry.x()).default();
