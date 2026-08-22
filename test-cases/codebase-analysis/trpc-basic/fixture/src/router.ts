import { procedure, router } from "./trpc";

export const appRouter = router({
  user: {
    get: procedure.query(() => null),
    create: procedure.input(z.string()).mutation(() => null),
  },
});
