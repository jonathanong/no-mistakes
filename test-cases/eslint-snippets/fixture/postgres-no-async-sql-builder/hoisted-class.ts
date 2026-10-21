// The class sits inside a hoisted function that is called before the const.
later();
const Q = "x";
async function build() {
  return Q;
}
function later() {
  class A {
    m() {
      build();
    }
  }
  new A().m();
}
