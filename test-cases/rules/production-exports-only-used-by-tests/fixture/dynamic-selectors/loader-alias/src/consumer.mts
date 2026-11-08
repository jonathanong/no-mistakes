const registry = { x: () => import("./worker.mjs") };
const alias = registry;
(await registry.x()).default();
