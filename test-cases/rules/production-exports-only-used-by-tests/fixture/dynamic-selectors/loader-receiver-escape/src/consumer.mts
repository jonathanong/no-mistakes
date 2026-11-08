const registry = { x: () => import("./worker.mjs"), expose: function () { return this; } };
(await registry.x()).default();
const alias = registry.expose();
alias.x().then(m => m.testOnly());
