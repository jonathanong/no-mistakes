const moduleSpecifier = "./targets/module.mts";

async function shadowed() {
  const moduleSpecifier = "./targets/shadowed.mts";
  const { selected } = await import(moduleSpecifier);
  selected();
  async function inner(moduleSpecifier: string) {
    await import(moduleSpecifier);
  }
  await inner("./targets/unknown.mts");
}

async function shadowedGlobals() {
  const URL = class { constructor(readonly value: string) {} };
  const String = (value: unknown) => value as string;
  await import(new URL("./targets/shadowed-url.mts", import.meta.url).value);
  await import(String(new URL("./targets/shadowed-string.mts", import.meta.url)));
}

let mutableSpecifier = "./targets/mutable-before.mts";
mutableSpecifier = "./targets/mutable-after.mts";
await import(mutableSpecifier);
await import(moduleSpecifier);
console.log(await import(moduleSpecifier));
export { shadowed, shadowedGlobals };
