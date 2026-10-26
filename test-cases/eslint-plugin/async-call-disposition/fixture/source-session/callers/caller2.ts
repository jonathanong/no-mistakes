import { enqueue } from '@session/jobs';
import { consume, awaiting } from '../shared/consumers';
const callback = (value: string) => enqueue(value);
consume('job', callback);
// The same owned consumer AST must also preserve a proven awaited callback.
const adopted = (value: string) => enqueue(value);
void awaiting('job', adopted);
