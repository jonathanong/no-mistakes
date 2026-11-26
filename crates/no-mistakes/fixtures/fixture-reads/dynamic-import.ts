consume(loadFixture("dynamic-import").label);
async function run(id: string) {
  const mod = await import("./loader");
  consume(mod.loadFixture(id).size);
}
