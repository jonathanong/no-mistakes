import Std, { ViaAlias, ViaArgument, ViaComputed, ViaMember } from "./namespace-consumer-lib";

declare function register(value: unknown): void;

const alias = ViaAlias;
export const aliased = () => new alias.ViaAliasDead();

register(ViaArgument);

export const computed = (name: "ViaComputedDead") => new ViaComputed[name]();

register(ViaMember.ViaMemberDead);

export const built = new Std.Built();
