declare module "pkg" { export function run(): string; }
declare module "legacy" { function run(): string; export = run; }
