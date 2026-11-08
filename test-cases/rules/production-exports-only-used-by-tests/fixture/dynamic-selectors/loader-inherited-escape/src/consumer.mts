const registry = { x: () => import("./worker.mjs") };
(await registry.x()).default();
const alias = registry.valueOf();
alias.x().then(m => m.testOnly());
