const Q = "x";
async function build() {
  return Q;
}
// `async` does not change hoisting. The default export can still run before Q.
export default async function () {
  build();
}
