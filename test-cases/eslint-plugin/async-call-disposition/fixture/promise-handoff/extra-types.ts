import { enqueueEmail } from '@app/jobs';
import { 'enqueueEmail' as literal } from './guard-target-literal';
import { enqueueEmail as missing } from './guard-target-missing';
import { enqueueEmail as overloaded } from './guard-target-overloaded';
import type { Dependencies } from './types';
export interface LiteralDependencies { x: typeof literal }
export interface MissingDependencies { x: typeof missing }
export interface OverloadedDependencies { x: typeof overloaded }
export interface ExtendedDependencies extends Dependencies { x: typeof enqueueEmail }
export interface QuotedDependencies { 'x': typeof enqueueEmail; other?(): void }
export interface MergedDependencies { x: typeof enqueueEmail }
export interface MergedDependencies { note?: string }
