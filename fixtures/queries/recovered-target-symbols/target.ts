export default function recovered() {
  return 'ok'
}

// Intentionally invalid at module scope. OXC recovery still retains the
// default-export symbol, which reverse queries must use.
return recovered
