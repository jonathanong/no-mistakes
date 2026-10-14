// The inline owner-history boundary remains conservative after a copied method.
import { primary } from '@vendor/valkey';
function replacement() {}
((client = primary) => { const alias = client; const get = alias.get; primary.get = replacement; get(); })();
