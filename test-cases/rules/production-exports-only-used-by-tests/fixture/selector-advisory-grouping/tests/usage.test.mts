import { loneExport } from '../src/single.mjs';
import { first, second, third } from '../src/shared.mjs';
import { tiedFirst, tiedSecond } from '../src/tied.mjs';
import { onlyTests } from '../src/definite.mjs';
loneExport(); first(); second(); third(); tiedFirst(); tiedSecond(); onlyTests();
