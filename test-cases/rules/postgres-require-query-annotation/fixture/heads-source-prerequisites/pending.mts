import { openTransaction } from './db.mjs';
import { statement } from './broken.mjs';
const tx = openTransaction();
tx(statement());
