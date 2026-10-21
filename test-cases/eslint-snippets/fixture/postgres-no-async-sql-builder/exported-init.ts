const Q = "x";
async function build() {
  return Q;
}
// An exported function is hoisted. A cyclic import can call init before Q.
export function init() {
  [1].map(() => {
    build();
  });
}
