const registry = { ...other, x: () => import("./worker.mjs") };
(await registry.x()).default();
