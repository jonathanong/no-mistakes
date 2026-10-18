import { primary, replica } from '@vendor/valkey';
// This binding has two possible values; neither call is an exact Known target.
for (const client of [primary, replica]) {
  client.get('iteration');
}
primary.info('primary sibling');
replica.info('replica sibling');
