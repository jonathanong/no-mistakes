import { primary, replica } from '@vendor/receiver';
primary.get = replica.get;
for (const item of ['a', 'b']) {
  primary.get(item);
}
