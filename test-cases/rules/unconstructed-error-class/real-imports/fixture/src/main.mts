import { Pool } from '@data-stores/psql/runtime';
import { Ajv } from 'ajv';
import * as AjvModule from 'ajv';
const AjvCtor = (AjvModule.default ?? AjvModule) as new () => unknown;
new Pool();
new Ajv();
new AjvCtor();
