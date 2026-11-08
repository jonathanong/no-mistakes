const registry = { x: () => { return import("./worker.mjs"); } };
(await registry.x()).default();
