import {write} from '@app/db';
function String(parts, ...values) { return parts.join(''); }
function through() { write(String`SELECT 2`); /* legacy-captured */ }
function shadow(String) { write(String`SELECT 3`); /* legacy-shadowed */ }
function other(parts) { return parts.join(''); }
write(String`SELECT 1`); // legacy-direct
through();
shadow(String);
shadow(other);
