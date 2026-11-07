const IDENTIFIERS = {
  github: {
    accountIdColumn: 'id',
    providerUserIdColumn: 'github_user_id',
    emailColumn: 'github_email',
  },
  gitlab: {
    accountIdColumn: 'account_id',
    providerUserIdColumn: 'gitlab_user_id',
    emailColumn: 'gitlab_email',
  },
} as const

export function getIdentifiers(provider: keyof typeof IDENTIFIERS) {
  return IDENTIFIERS[provider]
}
