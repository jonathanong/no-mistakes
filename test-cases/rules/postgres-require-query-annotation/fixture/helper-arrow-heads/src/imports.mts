import { read } from '@bench/db';
import { blockAlias, direct, conciseAlias } from './helpers.mjs';

read(blockAlias()); // One-hop local export aliases already preserve block-return heads.
read(direct());
read(conciseAlias()); // The same projection applies after concise-return classification.
