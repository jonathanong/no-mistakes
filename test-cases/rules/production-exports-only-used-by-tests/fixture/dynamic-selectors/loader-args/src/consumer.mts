const registry = { x: () => import("./worker.mjs") };
(await registry.x(1)).default();
