async function register() {
  const { test: check } = await require("vitest");
  check.skip("awaited", () => {});
}
