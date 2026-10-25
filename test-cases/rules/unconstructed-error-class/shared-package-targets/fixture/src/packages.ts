import { Pool } from '@fixture/pg-facade';
import { Pool as NamedPool } from 'pg';
import { Ajv } from 'ajv';
import * as AjvModule from 'ajv';
import { Box } from 'generic-helpers';
const AjvCtor = (AjvModule.default ?? AjvModule) as new () => unknown;
new Pool();
new NamedPool();
new Ajv();
new AjvCtor();
new Box<string>('value');
export class HelperSubclass extends Box<number> {}
export class PoolSubclass extends NamedPool {}
