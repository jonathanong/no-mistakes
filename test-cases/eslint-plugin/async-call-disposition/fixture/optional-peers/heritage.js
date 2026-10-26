import { enqueueEmail } from '@app/jobs';
import { consume } from './heritage-consumer';
const wrapped = () => enqueueEmail();
await consume(wrapped);
