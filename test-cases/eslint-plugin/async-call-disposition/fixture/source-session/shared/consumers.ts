// A dropped callback forces both the body proof and target-slot proof to fail.
export function consume(value: string, callback: (value: string) => Promise<void>) {
  callback(value);
}
