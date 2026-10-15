import { write } from './database.mts';
export async function begin() { return write; }
export async function client() { return { query: write }; }
