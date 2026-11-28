const source = "vitest";
const wrapper = require(source);
wrapper.each([[1]])("name", { sequential: true }, () => {});
