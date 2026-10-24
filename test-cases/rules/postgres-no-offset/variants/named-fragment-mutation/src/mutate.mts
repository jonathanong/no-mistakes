import { FRAGMENT } from "./fragments.mts";
// A different importer mutates the shared object before the caller executes it.
FRAGMENT.append(' OFFSET 100');
