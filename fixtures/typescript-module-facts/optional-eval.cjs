// Optional eval is indirect and cannot mutate this lexical scope.
const source = "var something = 1";
eval?.(source);
