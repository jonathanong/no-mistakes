// Module code is strict even without a directive.
function moduleFunction(value) {}
function moduleDefaults(value = "default") {}
const moduleArrow = () => {
  function moduleNested(value) {}
};
export { moduleFunction };
