export declare const lookalike: {
  getJob(id: string): Promise<{ isFailed(): Promise<boolean>; retry(): Promise<void> }>
  add(name: string, data: unknown): Promise<void>
}
