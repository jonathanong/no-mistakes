import { read } from '@bench/db';
import sql from 'sql-template-strings';
import { named, renamed, arrow, missing, changed, partial, early } from './helpers.mjs';
import type { named as typedNamed } from './helpers.mjs';
import { unavailable } from 'external-package';
import { forwarded } from './reexport.mjs';

read(named('ok'));
read(renamed('ok'));
read(arrow('ok'));
read(missing());
read(changed());
read(partial('maybe'));
read(early(true));
read(unavailable());
read(forwarded());
read(typedNamed('x'));
{
  const named = dynamic;
  read(named('shadow'));
}
read(named?.('optional'));
read(named(...parts));
read(await named('awaited'));
read(sql`/* direct */ SELECT 1`);
