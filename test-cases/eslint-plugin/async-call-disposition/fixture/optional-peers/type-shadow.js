import { enqueueEmail } from '@app/jobs';
import { alias, generic, iface, awaited, property, propertyAwaited } from './type-shadow-consumer';
// Erased declarations cannot shadow the runtime callback's dropped invocation.
const aliasJob = () => enqueueEmail();
await alias(aliasJob);
const genericJob = () => enqueueEmail();
await generic(genericJob);
const interfaceJob = () => enqueueEmail();
await iface(interfaceJob);
const awaitedJob = () => enqueueEmail();
await awaited(awaitedJob);
const propertyJob = () => enqueueEmail();
await property({ enqueueEmail: propertyJob });
const safePropertyJob = () => enqueueEmail();
await propertyAwaited({ enqueueEmail: safePropertyJob });
