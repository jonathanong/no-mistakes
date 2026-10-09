// Rebinding all arguments aliases must not reconnect a definitely deleted parameter.
function forward(parameter) {
  let slots = arguments;
  const deleted = delete slots[0];
  const escaped = opaque(slots);
  const reboundSlots = (slots = "replacement");
  const reboundArguments = (arguments = "replacement");
  const ignored = condition ? "one" : "two";
  return parameter;
}
const result = forward("/* disconnected parameter */ SELECT 1");
