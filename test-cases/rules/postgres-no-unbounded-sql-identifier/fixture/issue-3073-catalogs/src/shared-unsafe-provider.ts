const SHARED = { github: { emailColumn: 'github_email' } } as const
export function getShared(provider: keyof typeof SHARED) { return SHARED[provider] }
export function getSharedAgain(provider: keyof typeof SHARED) { return SHARED[provider] }
