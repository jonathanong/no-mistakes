// Named arguments bindings are legal in sloppy scripts; strict modules forbid them.
const forward = function arguments() { return arguments[0]; };
const result = forward('/* implicit arguments wins */ SELECT 1');
