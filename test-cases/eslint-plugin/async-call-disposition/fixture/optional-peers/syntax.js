import { enqueueEmail } from '@app/jobs';
import { consume } from './syntax-consumer';
const wrapped = () => enqueueEmail();
await consume(wrapped);
