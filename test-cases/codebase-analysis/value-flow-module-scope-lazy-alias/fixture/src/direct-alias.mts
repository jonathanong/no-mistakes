import { primary } from '@vendor/valkey';

function replacement() {}
const alias = primary;
primary.get = replacement;
// Direct module-scope aliases remain unresolved on the baseline and are not this follow-up.
alias.get('direct-control');
