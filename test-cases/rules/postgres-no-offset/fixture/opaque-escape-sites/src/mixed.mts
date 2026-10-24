import { read } from '@example/db';
declare const opaque: (executor: unknown) => unknown;
opaque( // LOW_SITE
  read, // MAX_SITE: retain this original reference span.
);
declare const statement: string;
read(statement); // DYNAMIC_SITE
