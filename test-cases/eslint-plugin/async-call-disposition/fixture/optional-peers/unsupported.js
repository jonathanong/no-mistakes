import { enqueueEmail } from '@app/jobs';
import { consume } from './unsupported/class';
const wrapped = () => enqueueEmail();
await consume(wrapped);
