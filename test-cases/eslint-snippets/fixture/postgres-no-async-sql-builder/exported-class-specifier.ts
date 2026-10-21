const Q = "x";
async function build() {
  return Q;
}
class A {
  m() {
    build();
  }
}
export { A };
new A().m();
