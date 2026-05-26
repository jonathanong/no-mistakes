import type { ReactNode } from "react";

interface Props {
  footer?: ReactNode;
  count?: number;
}

export function Sidebar({ footer, count }: Props) {
  const fallback = footer !== undefined ? footer : <DefaultFooter />;
  {
    const footer = "";
    footer ?? "fallback";
  }
  missing ?? "fallback";
  missingProps.footer ?? "fallback";
  return (
    <Panel footer={fallback} count={count ?? 0} />
  );
}

function DefaultFooter() {
  return null;
}
