import { primary, replica } from '@vendor/valkey';
for (const item of ['a', 'b']) {
  primary.get(item);
  primary.get = replica.get;
}
