// Returned callback frame remapping must retain fresh parameter metadata.
function factory(parameter) {
  const escaped = opaque(arguments);
  var parameter = '/* fresh annotation */ SELECT 1';
  return () => parameter;
}
const result = condition ? factory('placeholder') : '';
