import { DynamicConfig, ErrorConfig, BloomFilter, BloomError, Pool, PoolError, OpaqueConfig } from '@facade';
export class ConfigChild extends DynamicConfig {}
export class UnusedConfigError extends ErrorConfig {}
export class FilterChild extends BloomFilter {}
export class UnusedFilterError extends BloomError {}
export class PoolChild extends Pool {}
export class UnusedPoolError extends PoolError {}
export class UsedConfigError extends ErrorConfig {}
export class UsedFilterError extends BloomError {}
export class UsedPoolError extends PoolError {}
new UsedConfigError();
new UsedFilterError();
new UsedPoolError();
export class UnknownChild extends OpaqueConfig {}
