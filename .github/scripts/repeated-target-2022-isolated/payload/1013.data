import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
((client = primary) => { const alias = client; alias.get(); })();
