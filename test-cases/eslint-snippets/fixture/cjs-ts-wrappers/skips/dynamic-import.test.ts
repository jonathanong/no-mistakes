const moduleName = "./runner";
async function register() {
  const { test: check } = await import(moduleName);
  check.skip("dynamic import", () => {});
}
