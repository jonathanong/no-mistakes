import { primary } from '@vendor/valkey';
function replacement() {}
((client = primary) => { const alias = client; const get = alias.get; get(); })();
primary.get = replacement;
