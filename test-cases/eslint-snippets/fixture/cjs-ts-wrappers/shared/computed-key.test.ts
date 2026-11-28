const items = [];
const { ["test"]: check } = require("vitest");
check("case", () => {
  items.push(1);
});
