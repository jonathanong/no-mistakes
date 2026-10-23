const calls = [];
const client = {
  'foo%2Fbar': { create() { calls.push('percent'); } },
  'foo/responses': { create() { calls.push('slashed'); } },
};
client['foo%2Fbar'].create('input');
const percent = client['foo%2Fbar'];
percent.create('input');
client['foo/responses'].create('input');
const slashed = client['foo/responses'];
slashed.create('input');
calls;
