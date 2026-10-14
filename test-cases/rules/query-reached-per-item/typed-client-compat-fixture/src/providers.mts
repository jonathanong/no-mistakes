import { primary } from '@vendor/db';
export function createClient() { return { query: primary.query }; }
export function createArray() { return [primary.query]; }
export async function asyncClient() { return createClient(); }
export const exportedQuery = createClient().query;
export const exportedClient = createClient();
