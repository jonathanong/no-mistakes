import { primary as receiver, read } from '@vendor/plain-alternatives';
for (const item of ['a', 'b']) {
  receiver.transaction = read;
}
