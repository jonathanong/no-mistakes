const IDENTIFIERS = { github: { emailColumn: 'github_email' } } as const
export function getMemberIdentifiers(provider: keyof typeof IDENTIFIERS) {
  return IDENTIFIERS[provider]
}
