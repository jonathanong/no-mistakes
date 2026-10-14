import { primary, session } from '@vendor/valkey';
const items = ['one'];
// Inherited properties suppress defaults; explicit own undefined does not.
function inherited({ constructor: client = session }) { client.info('key'); }
for (const item of items) { inherited({}); }
function ownUndefined({ constructor: client = session }) { client.info('key'); }
for (const item of items) { ownUndefined({ constructor: undefined }); }
function ownClient({ constructor: client = primary }) { client.info('key'); }
for (const item of items) { ownClient({ constructor: session }); }
function inheritedNames({ toString: client = session }) { client.info('key'); }
for (const item of items) { inheritedNames({}); }
