async function register() {
  const { test: check } = await import("vitest" as string);
  check.skip("wrapped import", () => {});
}
