import { facade, unrelated } from './barrel.mts';
export function acquireBarrel() { return facade.get; }
export function acquireBarrelSibling() { return facade.set; }
export function acquireBarrelUnrelated() { return unrelated.get; }
