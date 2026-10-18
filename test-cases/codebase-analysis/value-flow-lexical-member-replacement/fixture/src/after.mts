import { primary, replica } from '@vendor/receiver';
for (const item of ['a', 'b']) { primary.get(item); }
primary.get = replica.get;
