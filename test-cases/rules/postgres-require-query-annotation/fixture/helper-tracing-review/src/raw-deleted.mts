import { write } from '@app/db';
delete String.raw;
delete String['raw'];
delete String['other'];
delete other.raw;
delete other['raw'];
delete object();
write(String.raw`/* deleted tag */ SELECT 1`); // unanalyzable:raw-property-delete
