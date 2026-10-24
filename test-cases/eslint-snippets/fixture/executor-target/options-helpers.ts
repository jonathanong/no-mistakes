import { read, type QueryExecutor } from "@example/db";
import type { QueryExecutor as ExecutorAlias } from "@example/db";
import type { QueryExecutor as OtherExecutor } from "./types";

export interface NamedOptions { query: QueryExecutor }
export type AliasOptions = { query: QueryExecutor };
export type RenamedOptions = { query: ExecutorAlias };
export interface ExtendedOptions extends NamedOptions { label?: string }
export type OptionalOptions = { query?: QueryExecutor };
export type StructuralOptions = { query: (statement: string) => unknown };
// A method's return type is not the type of the method value.
export interface MethodOptions { query(): QueryExecutor }
export type UnrelatedOptions = { query: OtherExecutor };
export type WrongOptions = { other: QueryExecutor };
export type UnionOptions = NamedOptions | WrongOptions;
// A mapped key has no explicit named property declaration.
export type MappedOptions = { [K in "query"]: QueryExecutor };
// Partial keeps the source declaration but makes the resolved property optional.
export type PartialOptions = Partial<NamedOptions>;
type LocalExecutor = QueryExecutor;
type AliasLayer = LocalExecutor;
export type AliasChainOptions = { query: AliasLayer };
type StructuralAlias = (statement: string) => unknown;
export type StructuralAliasOptions = { query: StructuralAlias };
type UnrelatedAlias = OtherExecutor;
export type UnrelatedAliasOptions = { query: UnrelatedAlias };
type CycleA = CycleB;
type CycleB = CycleA;
export type CycleOptions = { query: CycleA };
export type IntersectionOptions = { query: QueryExecutor } & { label?: string };
// Two declarations for query make the intersection's contract ambiguous.
export type AmbiguousOptions = { query: QueryExecutor } & { query: OtherExecutor };

export function named(options: NamedOptions) { options.query("SELECT 1"); }
export function alias(options: AliasOptions) { options.query("SELECT 1"); }
export function renamed(options: RenamedOptions) { options.query("SELECT 1"); }
export function inherited(options: ExtendedOptions) { options.query("SELECT 1"); }
export function optional(options: OptionalOptions) { options.query?.("SELECT 1"); }
export function structural(options: StructuralOptions) { options.query("SELECT 1"); }
export function method(options: MethodOptions) {}
export function unrelated(options: UnrelatedOptions) { options.query("SELECT 1"); }
export function wrong(options: WrongOptions) {}
export function plain(options) {}
export function union(options: UnionOptions) {}
export function mapped(options: MappedOptions) {}
export function partial(options: PartialOptions) {}
export function aliasChain(options: AliasChainOptions) { options.query("SELECT 1"); }
export function structuralAlias(options: StructuralAliasOptions) { options.query("SELECT 1"); }
export function unrelatedAlias(options: UnrelatedAliasOptions) { options.query("SELECT 1"); }
export function cycle(options: CycleOptions) {}
export function intersection(options: IntersectionOptions) { options.query("SELECT 1"); }
export function ambiguous(options: AmbiguousOptions) {}
export function destructuredNamed({ query }: NamedOptions) { query("SELECT 1"); }
export const destructuredAlias = ({ query: execute }: AliasOptions) => execute("SELECT 1");
export function destructuredDefault({ query = read }: NamedOptions) {}
export function destructuredComputed({ ["query"]: query }: NamedOptions) {}
export function rest(...options: NamedOptions[]) {}
export function overloaded(options: NamedOptions): void;
export function overloaded(options: unknown): void {}
export const contextual: (options: NamedOptions) => void = options => options.query("SELECT 1");
export let mutable = (options: NamedOptions) => options.query("SELECT 1");
export function written(options: NamedOptions) {}
written = options => options.query("SELECT hidden");
