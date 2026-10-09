import { write } from '@app/db';
function customTag() { return 'SELECT 1'; }
// Angle assertions are valid in .ts, but reserved in .mts/.cts.
(<unknown>String.raw) = customTag;
write(String.raw`/* asserted replacement */ SELECT 1`); // unanalyzable:raw-asserted-property
