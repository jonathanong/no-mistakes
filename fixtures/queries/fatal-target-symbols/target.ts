export default function fatal() {
  return 'unreachable'
}

// This declaration makes OXC report a fatal parser panic. The preceding
// partial AST must never be exposed as a valid default-export fact.
export const =
