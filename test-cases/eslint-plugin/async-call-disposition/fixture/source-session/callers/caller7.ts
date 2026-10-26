import { enqueue } from '@session/jobs';
import { consume } from '../shared/consumers';
const callback = (value: string) => enqueue(value);
consume('job', callback);
