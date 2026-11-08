const registry = { x: () => import("./worker.mjs") };
registry.x().default();
