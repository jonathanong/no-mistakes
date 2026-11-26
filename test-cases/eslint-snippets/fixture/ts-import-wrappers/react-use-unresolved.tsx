declare const reactModule: string;
const R = require(reactModule);
function A() {
  return R.use(Promise.resolve(1));
}
