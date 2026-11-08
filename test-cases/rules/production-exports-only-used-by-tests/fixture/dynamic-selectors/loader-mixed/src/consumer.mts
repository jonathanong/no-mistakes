const registry = { x: () => import("./worker.mjs") };
(await registry.x()).default();
consume(await registry.x());
