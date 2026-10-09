function forward(parameter) {
  const ignored = ([parameter] = ['SELECT 1']);
  return parameter;
}
const result = forward('/* fresh annotation */ SELECT 1');
