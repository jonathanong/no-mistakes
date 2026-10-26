import { enqueueEmail } from '@app/jobs';
import { enqueueEmail as declared } from '@app/declared-jobs';
import { enqueueEmail as ambient } from '@app/ambient-jobs';
import { enqueueEmail as synchronous } from '@app/non-promise-jobs';
import { enqueueEmail as shadowed } from '@app/shadowed-promise-jobs';
export interface Dependencies { x: typeof enqueueEmail; urlId?: string }
export interface DeclaredDependencies { x: typeof declared }
export interface AmbientDependencies { x: typeof ambient }
export interface SynchronousDependencies { x: typeof synchronous }
export interface ShadowedDependencies { x: typeof shadowed }
export type Job = typeof enqueueEmail;
export type AliasedDependencies = { x: Job };
