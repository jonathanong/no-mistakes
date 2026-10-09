// An arrow retains the enclosing fresh mapped parameter proof.
function forward(parameter) {
  const escaped = opaque(arguments);
  var parameter = '/* fresh annotation */ SELECT 1';
  const callback = () => parameter;
  return callback();
}
const result = forward('placeholder');
