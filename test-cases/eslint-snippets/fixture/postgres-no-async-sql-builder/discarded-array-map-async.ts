const Q = "x";
async function build() {
  return Q;
}

// The array is thrown away, so nothing observes the element promises.
async function discarded() {
  [1].map(() => build());
}

async function awaited() {
  await [1].map(() => build());
}

async function forwarded() {
  return build();
}

const arrow = async () => build();

async function mixed(flag: boolean) {
  return flag ? build() : 1;
}
