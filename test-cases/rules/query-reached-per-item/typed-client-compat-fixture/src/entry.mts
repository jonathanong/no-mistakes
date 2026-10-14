import { primary } from '@vendor/db';
import { createClient, createArray, asyncClient, exportedQuery, exportedClient } from './providers.mjs';
function arrayItems([query]) { query(); }
arrayItems(createArray());
async function promiseItems(client) { (await client).query(); }
promiseItems(asyncClient());
promiseItems(asyncClient());
function missingDefault(query = primary.query) { query(); }
missingDefault(createClient().missing);
function memberDefault({ missing = primary.query }) { missing(); }
memberDefault(createClient());
function boundItems(query, first, second) { query(first, second); }
boundItems(createClient().query.bind(null, 'a').bind(null, 'b'));
exportedQuery();
exportedClient.query();
// A promise is not a client until it is awaited.
function unawaitedItems(client) { client.query(); }
unawaitedItems(asyncClient());
unawaitedItems(asyncClient());
