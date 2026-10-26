import { enqueueEmail } from '@app/jobs';
import { parameterConsumer } from './parameter-guard-consumers';
declare const unproven: any;
// A parameter callee is not the enclosing function's implementation.
const job = (entries, options) => enqueueEmail(entries, options);
parameterConsumer(unproven, { x: job }, { x: enqueueEmail });
