import { Queue as Blocked } from './blocked.mts';
import { Queue as Missing } from './missing.mts';
import { Queue as Ambiguous } from './mixed.mts';
import Default from './adapter.mts';
import { absent } from './adapter.mts';
import { Queue as NoMap } from './no-map.mts';
import { Queue as Unresolved } from './unresolved-member.mts';
console.log(NoMap, Unresolved, Blocked, Missing, Ambiguous, Default, absent);
