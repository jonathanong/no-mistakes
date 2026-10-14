import { primary } from '@vendor/valkey';
function replacement() {}
primary.set = replacement;
(({ client }) => {
const alias = client; alias.get(); alias.set(); })({ client: primary });
