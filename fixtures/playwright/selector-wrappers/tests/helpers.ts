export type HelperType = unknown
export type OtherHelperType = unknown

export function getAsideLocator(page: unknown, testId: string, mode?: string) {
  return { page, testId, mode }
}

export function ambiguousLocator(page: unknown, testId: string) {
  return { page, testId }
}
