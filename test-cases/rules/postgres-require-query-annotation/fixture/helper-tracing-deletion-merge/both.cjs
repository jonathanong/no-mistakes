// Every branch deletes the slot, disconnecting the mapped parameter.
function forward(parameter) {
  const ignored = condition ? delete arguments[0] : delete arguments[0];
  const escaped = opaque(arguments);
  return parameter;
}
const result = forward('/* definite deletion */ SELECT 1');
