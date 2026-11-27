export function B0({ testId }: { testId?: string }) {
  return <button data-pw={testId} />;
}
export function B1({ testId }: { testId?: string }) {
  return <button data-pw={testId as string} />;
}
export function B2({ testId = "btn" as const }: { testId?: string }) {
  return <button data-pw={testId} />;
}
export function B3(props?: { testId?: string }) {
  const { testId = "btn" } = props!;
  return <button data-pw={testId} />;
}
