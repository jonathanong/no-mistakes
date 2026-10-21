const Q = "x";
async function build() {
  return Q;
}
// The two helpers call each other, so the const is not ordered.
function left() {
  right();
}
function right() {
  left();
  build();
}
left();
