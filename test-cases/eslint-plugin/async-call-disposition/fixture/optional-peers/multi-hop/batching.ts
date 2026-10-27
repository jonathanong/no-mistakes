export function sql(parts: TemplateStringsArray, ...values: unknown[]) {
  return { parts, values };
}

export async function processBatches(query: unknown, options: {
  handler: (rows: unknown[]) => Promise<void>;
}) {
  void query;
  await options.handler([]);
}
