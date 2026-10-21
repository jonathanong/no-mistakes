const Q = "x";
async function build() {
  return Q;
}
function later() {
  [1].map(() => {
    build();
  });
}
later();
