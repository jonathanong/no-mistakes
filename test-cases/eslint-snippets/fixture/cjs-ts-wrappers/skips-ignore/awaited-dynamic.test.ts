async function register() {
  const specifier = "vitest";
  const { test: check } = await require(specifier);
  check.skip("case", () => {});
}
