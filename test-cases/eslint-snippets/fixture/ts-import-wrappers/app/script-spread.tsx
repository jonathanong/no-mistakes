declare const props: { type?: string };

export const Spread = () => (
  <>
    <script type={"application/ld+json" as const} {...props} />
    <script {...props} type={"application/ld+json" as const} />
    <script type="application/ld+json" {...props} />
  </>
);
