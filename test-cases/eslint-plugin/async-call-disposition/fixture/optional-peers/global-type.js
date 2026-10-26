import { enqueueEmail } from '@app/jobs';
import { consume } from './global-type-consumer';
const wrapped = () => enqueueEmail();
await consume(wrapped);
