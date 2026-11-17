export function lifecycleFields(status: 'x' | 'y', at: Date): {
  cancelledAt: Date | null
  expiredAt: Date | null
  pastDueAt: Date | null
  pausedAt: Date | null
} {
  return {
    cancelledAt: status === 'y' ? at : null,
    expiredAt: null,
    pastDueAt: null,
    pausedAt: null,
  }
}
