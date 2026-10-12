export function safe() { return "/* safe */ SELECT 1"; }
export function unsafe() { return "SELECT 1"; }
export function shared() { return "/* shared */ SELECT 2"; }
export function ambiguous() { return "/* ambiguous */ SELECT 3"; }
export default function defaultQuery() { return "/* default */ SELECT 4"; }
