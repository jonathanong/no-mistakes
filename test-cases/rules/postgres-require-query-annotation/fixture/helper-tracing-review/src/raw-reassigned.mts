import { write } from '@app/db';
function customTag() { return 'SELECT 1'; }
String.raw = customTag;
String['raw'] = customTag;
(String.raw as unknown) = customTag;
(String.raw satisfies unknown) = customTag;
String.raw! = customTag;
String.fromCharCode = other;
write(String.raw`/* replaced tag */ SELECT ${value}`); // unanalyzable:raw-property-assignment
