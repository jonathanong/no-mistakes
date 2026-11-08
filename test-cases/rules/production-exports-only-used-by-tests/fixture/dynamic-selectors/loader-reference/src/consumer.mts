const registry = { x: () => import("./worker.mjs") };
const loader = registry.x;
