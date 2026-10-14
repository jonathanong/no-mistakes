import { client } from '@vendor/database';
client.query = replacement;
export default (client);
