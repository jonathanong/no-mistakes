const source = "vitest";
const wrapper = require(source);
wrapper()("name", { sequential: true }, () => {});
