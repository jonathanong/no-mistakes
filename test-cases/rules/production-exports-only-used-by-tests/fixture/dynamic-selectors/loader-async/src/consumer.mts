const registry = { x: async () => import("./worker.mjs") };
(await registry.x()).default();
