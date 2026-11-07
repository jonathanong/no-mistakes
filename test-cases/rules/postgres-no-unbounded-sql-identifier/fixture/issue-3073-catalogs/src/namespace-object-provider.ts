const IDENTIFIERS = { github: { emailColumn: 'github_email' } } as const
export function getObjectIdentifiers(provider: keyof typeof IDENTIFIERS) {
  return IDENTIFIERS[provider]
}
