// The callback is written before the const, so removing async would throw.
[1].map(() => build());
const Q = "x";
async function build() {
  return Q;
}
