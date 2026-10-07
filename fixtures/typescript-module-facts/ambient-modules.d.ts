// These exports belong to the ambient submodule, never this file.
declare module "*.txt" {
  import { type Shape } from "./external";
  const content: string;
  export default content;
  export { content as text };
  export { Shape } from "./external";
  export * from "./external";
  export const inline: string;
}
export const rootValue: string;
