const Q = "x";
async function build() {
  return Q;
}
// The callback sits inside the hoisted default export, so its own position
// is not readiness.
export default function () {
  [1].map(() => build());
}
