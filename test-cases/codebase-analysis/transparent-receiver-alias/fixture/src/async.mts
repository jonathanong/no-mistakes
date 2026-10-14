import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
(async (client = primary) => { const alias = client; alias.get(); })();
