export function schemaOf(value: string) {
  switch (value) {
    case "app":
      return "app";
    case "public":
      return "public";
    default:
      throw new Error("unsupported schema");
  }
}
