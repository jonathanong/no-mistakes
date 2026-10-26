
            import './side-effect';
            import type { Foo } from './types';
            import { used, unused } from './named';
            import defaultImport from './default';
            import * as namespaceImport from './namespace';
            import { onlyUnused } from './only-unused';
            defaultImport();
            namespaceImport.helper();
            used();
            fetch('/api/entry');
        