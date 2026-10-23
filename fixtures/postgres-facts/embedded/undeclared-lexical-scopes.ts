import type { PoolClient as Client, OtherClient } from '@example/db';
import { RuntimeClient } from '@example/db';

function generic<Client extends { query(sql: string): void }>(client: Client) {
  client.query('unrelated generic');
  const nested = (value: Client) => value.query('captured generic');
}
const arrow = <Client extends { query(sql: string): void }>(client: Client) => client.query('generic arrow');
function localAlias() {
  const client: Client = unrelated();
  client.query('alias declared later');
  type Client = { query(sql: string): void };
}
function localInterface() {
  interface Client { query(sql: string): void }
  const client: Client = unrelated();
  client.query('local interface');
}
function unionFallback<Client>(client: Client | OtherClient) {
  client.query('SELECT 1'); // diagnostic: unshadowed imported union member
}
function ordinary(client: Client) {
  client.query('SELECT 2'); // diagnostic: generic scope does not leak
  {
    var hoisted: Client = acquire();
    hoisted.query('SELECT 3'); // diagnostic: var binding owns function frame
    var inferred = new RuntimeClient();
    inferred.query('SELECT 4'); // diagnostic: inferred var in owning frame
  }
  hoisted.query('SELECT 5'); // diagnostic: annotation survives block
  inferred.query('SELECT 6'); // diagnostic: inferred identity survives block
  inferred = unrelated();
  inferred.query('unrelated replacement');
  {
    let hoisted = unrelated();
    hoisted.query('shadowed value');
  }
  hoisted.query('SELECT 7'); // diagnostic: inner shadow does not erase outer
}

class GenericClass<Client> {
  run(client: Client) { client.query('generic class'); }
}
function localClass() {
  class Client { query(sql: string) {} }
  const client: Client = new Client();
  client.query('local class');
}

function classExpression(client: Client) {
  const Local = class Client { run(value: Client) { value.query('class self'); } };
  client.query('SELECT 8'); // diagnostic: class expression self-name stays local
}
