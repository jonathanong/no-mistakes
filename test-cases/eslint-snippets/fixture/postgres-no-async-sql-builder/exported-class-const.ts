const Q = "x";
async function build() {
  return Q;
}
export const A = class {
  m() {
    build();
  }
};
new A().m();
