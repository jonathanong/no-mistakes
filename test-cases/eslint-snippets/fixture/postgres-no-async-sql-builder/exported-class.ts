const Q = "x";
async function build() {
  return Q;
}
export class A {
  m() {
    build();
  }
}
new A().m();
