import { enqueueAmbiguous, enqueueUnknown, enqueueUnion, enqueueGeneric, enqueueShadowed } from './control-targets';
function ambiguous(callback: typeof enqueueAmbiguous): typeof enqueueAmbiguous { return callback; }
function unknown(callback: typeof enqueueUnknown): typeof enqueueUnknown { return callback; }
function union(callback: typeof enqueueUnion): typeof enqueueUnion { return callback; }
function generic(callback: typeof enqueueGeneric): typeof enqueueGeneric { return callback; }
function shadowed(callback: typeof enqueueShadowed): typeof enqueueShadowed { return callback; }
void ambiguous(() => enqueueAmbiguous());
void unknown(() => enqueueUnknown());
void union(() => enqueueUnion());
void generic(() => enqueueGeneric());
void shadowed(() => enqueueShadowed());
