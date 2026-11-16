import { query } from '@example/db';
const ENV = [process.env.TABLE].map((name) => `COUNT(${name})`).join(',');
const CALL = [readTable()].map((name) => `COUNT(${name})`).join(',');
const SPREAD = [...readTables()].map((name) => `COUNT(${name})`).join(',');
const CAPTURED = ['orders', 'refunds'].map(() => `COUNT(${process.env.COLUMN})`).join(',');
const OPAQUE = ['orders', 'refunds'].map((name) => `COUNT(${this.column})`).join(',');
const HOLE = ['orders', , 'refunds'].map((name) => `COUNT(${name})`).join(',');

export function environment() { return query(`SELECT ${ENV} FROM orders`); }
export function call() { return query(`SELECT ${CALL} FROM orders`); }
export function spread() { return query(`SELECT ${SPREAD} FROM orders`); }
export function captured() { return query(`SELECT ${CAPTURED} FROM orders`); }
export function opaque() { return query(`SELECT ${OPAQUE} FROM orders`); }
export function hole() { return query(`SELECT ${HOLE} FROM orders`); }
