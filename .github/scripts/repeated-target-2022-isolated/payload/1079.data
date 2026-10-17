const { primary } = require('@vendor/valkey');
function replacement() {}
// Keep sloppy-script Annex B initializers conservative, even though eager.
for (var key = ((client = primary) => { const get = client.get; primary.get = replacement; get(); return 1; })() in { a: 1 }) {}
