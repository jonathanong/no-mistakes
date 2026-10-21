const Q = "x";
async function build() {
  return Q;
}
class A {
  m = () => {
    build();
  };
}
new A().m();
