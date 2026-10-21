// The method text is after the const, but new A() runs before it.
new A().m();
const Q = "x";
async function build() {
  return Q;
}
class A {
  m() {
    build();
  }
}
