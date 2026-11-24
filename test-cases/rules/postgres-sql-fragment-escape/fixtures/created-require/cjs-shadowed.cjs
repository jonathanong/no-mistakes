function shadowed(require) {
  const { createRequire: shadowMake } = require("module");
  const shadowLoad = shadowMake(__filename);
  shadowLoad("../shadowed.cjs");
}
shadowed(() => ({ createRequire: () => () => {} }));
