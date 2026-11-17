import SQL from 'sql-template-strings';
import { query } from '@example/db';
import { NUMBER_VALUE, STRING_VALUE } from './constants.mts';
import * as constants from './constants.mts';

export function projectedValues() {
  return query(SQL`SELECT ${NUMBER_VALUE}, ${STRING_VALUE} FROM entries OFFSET 1`);
}

export function conditionalValues() {
  return query(SQL`SELECT CASE WHEN true THEN ${NUMBER_VALUE} ELSE ${STRING_VALUE} END FROM entries OFFSET 1`);
}

export function namespaceValues() {
  return query(SQL`SELECT ${constants.NUMBER_VALUE}, CASE WHEN true THEN ${constants.STRING_VALUE} ELSE ${constants.NUMBER_VALUE} END FROM entries OFFSET 1`);
}

const PUNCTUATION_VALUE = '-suffix';
export function punctuationValues() {
  return query(SQL`SELECT ${PUNCTUATION_VALUE}, CASE WHEN true THEN ${PUNCTUATION_VALUE} ELSE ${PUNCTUATION_VALUE} END FROM entries OFFSET 1`);
}

const EMPTY_VALUE = '';
export function emptyValues() {
  return query(SQL`SELECT ${EMPTY_VALUE}, CASE WHEN true THEN ${EMPTY_VALUE} ELSE ${PUNCTUATION_VALUE} END FROM entries OFFSET 1`);
}
