const Q = "x";
async function build() {
  return Q;
}
new (class {
  m() {
    build();
  }
})().m();
