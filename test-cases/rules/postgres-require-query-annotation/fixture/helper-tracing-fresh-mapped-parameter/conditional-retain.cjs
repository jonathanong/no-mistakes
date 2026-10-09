// Both pure branches preserve the existing fresh parameter binding.
function forward(parameter) {
  const escaped = opaque(arguments);
  var parameter = '/* fresh annotation */ SELECT 1';
  const ignored = condition ? 'one' : 'two';
  return parameter;
}
const result = forward('placeholder');
