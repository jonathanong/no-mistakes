import { enqueueEmail } from '@app/jobs';
import { consume } from './unsupported/enum';
const wrapped = () => enqueueEmail();
await consume(wrapped);
