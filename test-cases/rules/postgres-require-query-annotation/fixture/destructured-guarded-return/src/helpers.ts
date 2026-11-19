export function parseId(value: string): string | undefined {
  try {
    const url = new URL(value)
    for (const segment of url.pathname.split('/')) {
      if (segment.startsWith('w-')) return segment.slice(2)
    }
  } catch {
    return undefined
  }
  return undefined
}

export function isId(value: string): boolean {
  switch (value.length) {
    case 8:
      return /^[0-9a-f]+$/.test(value)
    default:
      return false
  }
}
