import { enqueueEmail } from '@app/jobs';
import { consume } from './global-value-consumer';
const wrapped = () => enqueueEmail();
await consume(wrapped);
