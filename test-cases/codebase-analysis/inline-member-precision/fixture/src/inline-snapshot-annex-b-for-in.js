const { primary } = require('@vendor/valkey');
function replacement() {}
// Retain the existing conservative Annex B initializer boundary.
for (var key = ((client = primary) => {
  const get = client.get;
  primary.get = replacement;
  get();
  return 'key';
})() in { first: 1, second: 2 }) {}
