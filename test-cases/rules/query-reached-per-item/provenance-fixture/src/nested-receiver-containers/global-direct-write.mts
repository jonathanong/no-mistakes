import { primary as client } from '@vendor/valkey';
client.get = function replacement() {};
