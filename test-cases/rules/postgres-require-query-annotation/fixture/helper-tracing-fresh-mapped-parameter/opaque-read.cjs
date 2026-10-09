// Construction reads an immutable parameter; it does not assign that binding.
function forward(parameter) {
  const copy = new Box(parameter);
  return parameter;
}
const result = forward('/* fresh annotation */ SELECT 1');
