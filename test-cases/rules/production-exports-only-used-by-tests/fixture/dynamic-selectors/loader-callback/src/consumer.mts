const registry = { x: () => import("./worker.mjs") };
consume(registry.x);
