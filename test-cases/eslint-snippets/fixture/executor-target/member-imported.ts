import { read } from '@example/db';
import * as helpers from './member-api';
import { mutable } from './member-api';

helpers.load(read);
helpers['arrow'](read);
helpers.dynamic(read, 'SELECT id FROM t');
mutable.load(read);
const alias = helpers;
alias.load(read);
helpers.options({ query: read });
