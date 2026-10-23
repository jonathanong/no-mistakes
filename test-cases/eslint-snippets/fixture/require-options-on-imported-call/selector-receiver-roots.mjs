const client = buildClient();
const request = client.responses.create;
request('input');
const responses = client.responses;
responses.create('input');
// Receiver selectors assert lexical shape even for local or parameter roots.
function scoped(client) {
  const request = client.responses.create;
  request('input');
}
