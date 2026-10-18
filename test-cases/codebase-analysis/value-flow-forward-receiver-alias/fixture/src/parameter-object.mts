import { replica, untouched } from '@vendor/forward-parameter-object';
function replacement() {}
function write(x) { x.client.get = replacement; }
write({ client: replica });
replica.get('changed');
untouched.get('control');
