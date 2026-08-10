// Oxc 0.143 parses inline exports separately from local export specifiers.
export function InlineFunction() {
  return <main />;
}

export const InlineArrow = () => <aside />;
