import { primary } from '@vendor/valkey';
function replacement() {}
// Capturing a receiver retains a live member read, unlike a copied method.
export default ((client = primary) => { const alias = client; primary.get = replacement; return () => alias.get(); })();
