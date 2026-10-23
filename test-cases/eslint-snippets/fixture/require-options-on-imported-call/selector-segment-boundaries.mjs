const calls = [];
const client = {
  foo: { responses: { create() { calls.push('nested'); } } },
  'foo.responses': { create() { calls.push('dotted'); } },
  'foo/responses': { create() { calls.push('slashed'); } },
};
// These two keys name single properties, not the nested foo.responses API.
client['foo.responses'].create('input');
const dotted = client['foo.responses'];
dotted.create('input');
client['foo/responses'].create('input');
const slashed = client['foo/responses'];
slashed.create('input');
client.foo.responses.create('input');
calls;
