import { Entrypoint, RuntimeError } from 'fixture:runtime';
import { DirectBase, DirectError } from 'fixture:direct';
export class App extends Entrypoint {}
export class UnusedRuntime extends RuntimeError {}
export class UsedRuntime extends RuntimeError {}
new UsedRuntime();
export class DirectChild extends DirectBase {}
export class UnusedDirect extends DirectError {}
