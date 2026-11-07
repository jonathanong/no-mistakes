import { read } from '@bench/db';

const NAME = 'accounts/list';
read(`/* ${NAME} */ SELECT 1`);
function local() {
  const name = 'sessions/list';
  read(`/* ${name} */ SELECT 2`);
}
read('SELECT 3');
const EMPTY = '';
read(`/* ${EMPTY} */ SELECT 4`);
const TERMINATOR = '*/';
read(`/* ${TERMINATOR} */ SELECT 5`);
let mutable = 'accounts/list';
read(`/* ${mutable} */ SELECT 6`);
const alias = NAME;
read(`/* ${alias} */ SELECT 7`);
function shadow(NAME: string) {
  read(`/* ${NAME} */ SELECT 8`);
}
export const EXPORTED = 'accounts/list';
read(`/* ${EXPORTED} */ SELECT 9`);
