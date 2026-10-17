import { primary } from '@vendor/valkey';
function replacement() {}
(async (client) => { const alias = client; await Promise.resolve(); alias.get(); })(primary);
primary.get = replacement;
