function forward(callback) {
  const original = arguments;
  const escaped = opaque(original);
  var callback = () => {
    const assigned = (arguments[0] = '/* re-escaped value */ SELECT 1');
    const escapedAgain = opaque(original);
  };
  // The installed callback's later escape revokes assignment-time freshness.
  return callback;
}
const result = forward('initial');
