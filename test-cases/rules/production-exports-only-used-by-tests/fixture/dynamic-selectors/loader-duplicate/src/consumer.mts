const registry = { x: () => import("./worker.mjs"), x: other };
(await registry.x()).default();
