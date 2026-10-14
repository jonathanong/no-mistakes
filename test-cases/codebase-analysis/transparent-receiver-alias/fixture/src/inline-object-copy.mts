import { primary } from '@vendor/valkey';
function replacement() {}
primary.get = replacement;
((client = { get: primary.get }) => { const alias = client; alias.get(); })();
