async function observedThroughOriginal() {
  const p = work();
  const alias = p;
  p.catch(() => {});
  await barrier();
  await expect(alias).rejects.toThrow();
}

async function observedThroughAlias() {
  const p = work();
  const alias = p;
  alias.catch(() => {});
  await barrier();
  await expect(p).rejects.toThrow();
}

async function observedThroughChain() {
  const p = work();
  const one = p;
  const two = one;
  two.catch(() => {});
  await barrier();
  await expect(p).rejects.toThrow();
}

async function earlyMatcherThroughAlias() {
  const p = work();
  const alias = p;
  const matcher = expect(alias).rejects.toThrow();
  await barrier();
  await matcher;
  await expect(p).rejects.toThrow();
}

async function mutableAlias() {
  const p = work();
  let alias = p;
  await barrier();
  await expect(alias).rejects.toThrow();
}

// A write, even to a syntactic const, prevents alias identity proof.
async function writtenAlias() {
  const p = work();
  const alias = p;
  alias = other();
  await barrier();
  await expect(alias).rejects.toThrow();
}

async function mutableOrigin() {
  let p = work();
  const alias = p;
  await barrier();
  await expect(alias).rejects.toThrow();
}

async function shadowedAlias() {
  const p = work();
  const alias = p;
  {
    const alias = other();
    alias.catch(() => {});
  }
  await barrier();
  await expect(alias).rejects.toThrow();
}

async function forwardReference() {
  const alias = p;
  const p = work();
  await barrier();
  await expect(alias).rejects.toThrow();
}

async function cyclicAliases() {
  const one = two;
  const two = one;
  await barrier();
  await expect(one).rejects.toThrow();
}

async function conditionalValue(flag: boolean) {
  const p = work();
  const alias = flag ? p : other();
  alias.catch(() => {});
  await barrier();
  await expect(p).rejects.toThrow();
}

// The alias initializer runs while p is still in its temporal dead zone.
async function reversedStatementAlias() {
  const alias = p,
    p = work();
  await barrier();
  await expect(alias).rejects.toThrow();
}
