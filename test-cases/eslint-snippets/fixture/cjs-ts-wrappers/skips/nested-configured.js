const {
  helpers: { test: check },
} = require("./custom-test");
check.skip("nested configured export", () => {});
const loaded = require("./custom-test");
const {
  helpers: { test: alias },
} = loaded;
alias.skip("nested configured alias", () => {});
