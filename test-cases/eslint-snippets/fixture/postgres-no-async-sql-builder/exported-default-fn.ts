const Q = "x";
async function build() {
  return Q;
}
// A default function declaration is hoisted and has no binding.
// A cyclic import can call it before Q.
export default function () {
  build();
}
