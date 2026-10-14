import { primary } from '@vendor/valkey';
function replacement() {}
(async (client = primary) => { const alias = client; await Promise.resolve(); alias.get(); })();
primary.get = replacement;
