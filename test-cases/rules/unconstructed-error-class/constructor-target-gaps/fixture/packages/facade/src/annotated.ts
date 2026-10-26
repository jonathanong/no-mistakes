import { RawConfig, RawError } from 'raw-constructors';
export const DynamicConfig: new (...args: any[]) => RawConfig = RawConfig;
export const ErrorConfig: new (...args: any[]) => RawError = RawError;
declare const opaque: any;
// An annotation alone is not runtime identity proof.
export const OpaqueConfig: new () => RawError = opaque;
export const BroadlyAnnotated: new () => object = RawConfig;
