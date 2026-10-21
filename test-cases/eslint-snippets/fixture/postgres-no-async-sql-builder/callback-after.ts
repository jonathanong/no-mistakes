const Q = "x";
async function build() {
  return Q;
}
// The issue example. The callback returns the call, and map's result is discarded.
[1].map(() => build());
