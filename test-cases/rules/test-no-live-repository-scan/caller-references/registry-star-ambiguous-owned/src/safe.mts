export function processSync(root: string) {
  return root;
}

export const workers = { sync: processSync };
