import { enqueueEmail } from '@app/jobs';
import { staticForward } from './static-forward';
const job: typeof enqueueEmail = (entries, options) => enqueueEmail(entries, options);
staticForward({ x: job });
