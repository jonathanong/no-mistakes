// This runtime edge must stay outside transitive annotation eligibility:
// src/a imports only our type, so admitting it would widen beyond runtime flow.
import { noise } from '@p/noise';
export interface Marker { value: string }
export const unused = noise;
