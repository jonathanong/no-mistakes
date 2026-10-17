import { primary as client, replica } from '@vendor/valkey';
function replacement() {}
const before = client.get;
{
  // A local receiver and its imported member occupy distinct lexical scopes.
  let box = { client };
  box.client.get = replacement;
}
before('copied');
function consume(value) { value.get('changed'); }
consume(client);
replica.get('untouched');
