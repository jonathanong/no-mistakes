// later is hoisted and called before the const. The callback's source position
// is after the const, but the call runs first.
later();
const Q = "x";
async function build() {
  return Q;
}
function later() {
  [1].map(() => {
    build();
  });
}
