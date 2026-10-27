// Type queries retain their legacy type-only flag after resolving a value binding.
declare const arguments: unknown;
function query() { type Capture = typeof arguments; return arguments; }
type Missing = arguments;
import Legacy = arguments.Value;
