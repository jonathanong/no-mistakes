import { primary, replica } from '@vendor/valkey';
import { object, array, copied } from './provenance-factories.mjs';
function replacement() {}
// Imported factories wrap these lazy containers in returned-member provenance.
const options = object(primary);
const clients = array(replica);
const snapshot = copied(primary);
function objectBefore(value) { value.client.get('key'); }
function arrayBefore(value) { value['0'].get('key'); }
objectBefore(options);
arrayBefore(clients);
primary.get = replacement;
replica.get = replacement;
function objectAfter(value) { value.client.get('key'); }
function arrayAfter(value) { value['0'].get('key'); }
function copiedAfter(value) { value.get('key'); }
objectAfter(options);
arrayAfter(clients);
copiedAfter(snapshot);
