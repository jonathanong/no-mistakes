// Empty intermediate scopes must not hide the nearest ancestor binding.
const x = 0;
function outer() {
  { const x = 1; console.log(x); }
  const outerName = 1;
  function inner() {
    { const outerName = 2; console.log(outerName); }
    { const neverShadowed = 3; console.log(neverShadowed); }
  }
}
