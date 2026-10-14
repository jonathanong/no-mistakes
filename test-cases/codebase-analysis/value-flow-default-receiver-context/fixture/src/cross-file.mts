import { make } from './source.mjs';
//----------------------------------------------------------------------------------------
function run({ client } = make()) { client.get('key'); }
run();
