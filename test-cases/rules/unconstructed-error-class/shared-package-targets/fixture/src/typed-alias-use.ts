import { Configured } from '@fixture/config';
import { Filter } from '@fixture/filter';
new Configured({ key: 'primary', enabled: true });
new Filter<string>('value');
