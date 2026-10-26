import { enqueueEmail } from '@app/jobs';
import { consume } from './global-type-import-consumer';
const wrapped = () => enqueueEmail();
await consume(wrapped);
