// A later container escape invalidates the fresh direct parameter assignment.
function forward(parameter) {
  const escaped = opaque(arguments);
  var parameter = '/* fresh annotation */ SELECT 1';
  const escapedAgain = opaque(arguments);
  return parameter;
}
const result = forward('placeholder');
