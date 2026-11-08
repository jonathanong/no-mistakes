const registry = { x: () => import("./worker.mjs") };
consume(registry);
(await registry.x()).default();
