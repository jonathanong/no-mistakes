import {write} from '@app/db';
function String(parts) { return 'SELECT 6'; }
write(String`SELECT 7`); // legacy-static-return-untrusted
