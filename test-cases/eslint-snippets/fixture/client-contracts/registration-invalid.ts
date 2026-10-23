import { register } from '@app/lifecycle';
const onStop = () => {};
function later() { register(onStop); }
if (enabled) register(onStop);
register(other);
register?.(onStop);
register(onStop, extra);
ordinary();
