import {write} from '@app/db';
async function String(parts, ...values) { return parts.join(''); }
write(String`SELECT 4`); // legacy-async-promise
write(await String`SELECT 5`); // legacy-async-awaited
