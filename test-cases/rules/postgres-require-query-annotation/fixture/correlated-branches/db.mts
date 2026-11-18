export type Run = (statement: unknown) => void

export const read: Run = (statement) => {
  void statement
}
