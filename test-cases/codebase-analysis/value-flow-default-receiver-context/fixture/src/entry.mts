import { primary as late, session as early, replica as external, scoped } from '@vendor/valkey';
function replacement() {}
function beforeWrite(client = late) {
  client.get('key');
  late.get = replacement;
}
function afterWrite(client = early) {
  early.get = replacement;
  client.get('key');
}
function externalWrite(client = external) {
  client.get('key');
}
function objectDefault({ client } = { client: scoped }) {
  client.get('key');
  scoped.get = replacement;
}
external.get = replacement;
beforeWrite();
afterWrite();
externalWrite();
objectDefault();
