import { session } from './clients.mjs';
import { readLocalLatch, localDiagnostics } from './local-client-helpers.mjs';
const items = ['one'];
for (const item of items) { readLocalLatch(); }
for (const item of items) { localDiagnostics(session); }
