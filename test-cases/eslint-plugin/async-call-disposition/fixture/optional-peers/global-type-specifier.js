import { enqueueEmail } from '@app/jobs';
import { consume } from './global-type-specifier-consumer';
const wrapped = () => enqueueEmail();
await consume(wrapped);
