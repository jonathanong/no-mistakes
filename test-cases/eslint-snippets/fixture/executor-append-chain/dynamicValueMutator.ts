function use(ns: unknown) {
  return ns;
}

export async function load() {
  const loaded = await import("./dynamicValueControls");
  use(loaded);
}
