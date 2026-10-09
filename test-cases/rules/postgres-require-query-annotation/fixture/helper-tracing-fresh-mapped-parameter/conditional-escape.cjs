// A container escape on either branch cancels the old fresh binding proof.
function forward(parameter) {
  const escaped = opaque(arguments);
  var parameter = '/* fresh annotation */ SELECT 1';
  const ignored = condition ? opaque(arguments) : false;
  return parameter;
}
const result = forward('placeholder');
