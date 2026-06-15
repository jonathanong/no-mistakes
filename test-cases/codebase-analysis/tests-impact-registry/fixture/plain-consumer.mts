// Ordinary importer of the target — NOT a registry, so it must not produce a
// registry hint.
import { feature } from './feature.mts';

export const usePlainFeature = () => feature();
