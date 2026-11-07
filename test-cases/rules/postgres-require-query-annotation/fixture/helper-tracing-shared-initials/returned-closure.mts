function outer() {
  function inner() {
    return "/* nested */ SELECT 1";
  }
  return inner;
}
export const queryText = outer();
