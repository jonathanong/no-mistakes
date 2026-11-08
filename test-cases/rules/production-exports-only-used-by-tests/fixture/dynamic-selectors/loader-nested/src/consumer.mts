const registry = { group: { x: () => import("./worker.mjs") } };
(await registry.group.x()).default();
