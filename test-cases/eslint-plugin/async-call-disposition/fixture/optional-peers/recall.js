import { enqueueEmail } from '@app/jobs';
import { drop, opaque, dropProperties, shadowed, defaultShadow } from './recall-consumer.js';
const dropped = () => enqueueEmail();
drop(dropped);
const escaped = () => enqueueEmail();
opaque(escaped);
const unknown = () => enqueueEmail();
unknownConsumer(unknown);
const property = () => enqueueEmail();
dropProperties({ enqueueEmail: property });
// Awaiting a shadowed parameter cannot discharge the outer callback.
const shadow = () => enqueueEmail();
await shadowed(shadow);
// Parameter defaults run before the function body var environment exists.
const parameterDefault = () => enqueueEmail();
await defaultShadow(parameterDefault);
