import { execute as left } from './left/execute.mjs';
import { execute as right } from './right/execute.mjs';
left('SELECT left_value');
right('SELECT right_value');
// Same-spelled functions keep separate module identities across callback arms.
const callback = unknownCondition
  ? () => left('SELECT left_value')
  : () => right('SELECT right_value');
opaqueConsumer(callback);
