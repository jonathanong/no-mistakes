import { API, ImportedRoot } from 'namespaces';
export class DirectError extends API.Child {}
export class NestedError extends API.Inner.Child {}
export class InnerNonError extends API.Own.Child {}
export class ImportedNonError extends ImportedRoot {}
export class OpaqueBase extends API.Opaque.Child {}
import { Benign } from 'namespaces';
export class OpaqueShadow extends Benign.Inner.Child {}
import { Split, Shared } from 'namespaces';
export class AmbientMergedNonError extends Split.Child {}
export class MergedExportedError extends Shared.Child {}
import { AmbientMerged } from 'namespaces';
export class AmbientMergedError extends AmbientMerged.Child {}
