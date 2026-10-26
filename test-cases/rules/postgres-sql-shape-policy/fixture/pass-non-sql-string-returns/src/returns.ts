export function sign(): string {
  return '-'
}

export function moduleExtension(): string {
  return '.mjs'
}

export function commonExtension(): string {
  return '.cjs'
}

export function inactive(): string {
  return 'Profile not active'
}

export function stringifyError(): string {
  return 'Failed to stringify error'
}

export function ok(): string {
  return 'ok'
}

export function emptyTemplate(): string {
  return `ok`
}

export const arrowSign = (): string => {
  return '-'
}

export function unusedWrapper(): string {
  return sign()
}

export const ignored = [1].map(() => {
  return '-'
})

export function formatDuration(seconds: number): string {
  return `${seconds}s`
}
