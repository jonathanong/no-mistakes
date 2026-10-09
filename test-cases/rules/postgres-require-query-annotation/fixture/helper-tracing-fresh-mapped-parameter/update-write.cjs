function forward(parameter) {
  const ignored = parameter++;
  return parameter;
}
const result = forward('/* fresh annotation */ SELECT 1');
