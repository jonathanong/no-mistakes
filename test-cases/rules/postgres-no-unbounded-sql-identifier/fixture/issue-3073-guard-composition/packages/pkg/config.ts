export const providers = ["facebook", "github"] as const
export type Provider = (typeof providers)[number]
export const regions = ["north", "south"] as const
export type Region = (typeof regions)[number]
