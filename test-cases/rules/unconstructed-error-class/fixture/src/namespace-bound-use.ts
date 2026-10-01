import { Handed } from "./namespace-bound";

export const handed = new (Handed.HandedDead.bind(null))();
