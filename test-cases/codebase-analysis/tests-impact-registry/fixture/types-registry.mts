// A registry-shaped file that only TYPE-imports the target. A type-only
// reference is not a runtime registration, so it must NOT produce a hint.
import type { Feature } from './feature.mts';

export type RegisteredFeature = Feature;
