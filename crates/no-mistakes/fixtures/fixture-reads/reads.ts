function readFields(key: string) {
  consume(loadFixture("direct").size);
  consume(loadFixture("string")["a/b~c"]);
  consume(loadFixture("number")[0]);
  consume(loadFixture("wrapped")?.size);
  consume((loadFixture("asserted") as Fixture)!.size);
  consume((<Fixture>loadFixture("type-asserted")).size);
  consume((loadFixture("satisfies") satisfies Fixture).size);
  consume(loadFixture("large-number")[1e12]);
  consume(loadFixture("dynamic")[key]);
  consume(loadFixture("prototype").constructor);
  consume(loader.fetch("member").size);
  consume(loader?.fetch("optional-callee").size);
  consume(loadFixture?.("optional-call").size);
  consume(loadFixture("two-args", key).size);
  consume((loadFixture)("wrapped-callee").size);

  const local = loadFixture("alias");
  type FixtureType = typeof local;
  const nested = local.details;
  consume(nested.size);

  const { ["a/b~c"]: label, 0: first } = loadFixture("destructure");
  consume(label, first);

  const { details: { size = 0 } } = loadFixture("default");
  consume(size);

  const { [key]: computed } = loadFixture("computed-binding");
  consume(computed);

  const { ...rest } = loadFixture("rest");
  consume(rest);

  const [entry] = loadFixture("array-pattern");
  consume(entry);

  const {} = loadFixture("empty-object-pattern");
  const unused = loadFixture("unused-root");
  const unusedField = loadFixture("unused-field").size;

  let mutable = loadFixture("mutable");
  consume(mutable.size);
  const reassigned = loadFixture("write-alias");
  reassigned = {};
  consume(reassigned.size);

  loadFixture("write-field").size = 7;
  delete loadFixture("delete-field").size;
  loadFixture("increment").size++;
  loadFixture("method").details.format();

  const fn = new (loadFixture("construct").make)();
  consume(fn);
  const copy = { ...loadFixture("spread") };
  consume(copy);
  for (const entry of loadFixture("iterate")) consume(entry);
  for (const field in loadFixture("enumerate")) consume(field);

  consume(loadFixture("whole"));
  consume(loadFixture("whole").label);
  consume(loadFixture("alternate").label);
  consume(loadFixture( 'alternate' ).size);
  consume(loadFixture("repeated").label, loadFixture("repeated").size);

  // Another occurrence of loadFixture("source-only") might be a generated call.
  consume(loadFixture("source-only").label);
}

export const shared = loadFixture("exported");
export default loadFixture("export-default");

async function readAsync() {
  return await loadFixture("awaited");
}
