import { enqueue } from '@bench/jobs';
import { consume } from '../shared/consumers';
import type { Payload } from '../shared/contracts';
const payload: Payload = { id: 'job', count: 1 };
const callback = (value: Payload) => {
  void value.id;
  return enqueue(value);
};
consume(payload, callback);
