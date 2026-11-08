const registry = { x: () => import("./worker.mjs"), y: () => import("./other.mjs") };
(await registry.x()).default();
(await registry.y()).other();
