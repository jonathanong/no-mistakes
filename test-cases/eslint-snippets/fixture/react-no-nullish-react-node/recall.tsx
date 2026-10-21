import type { ReactNode } from "react";

interface Props {
  children?: ReactNode;
  footer?: React.ReactNode;
}

declare const name: string;

function Empty() {
  return null;
}

export function Layout(props: Props) {
  const node = props.children;
  const { footer } = props;
  const slot = footer;
  return (
    <section>
      {props["children"] ?? <Empty />}
      {props[`footer`] ?? <Empty />}
      {node ?? <Empty />}
      {slot ?? <Empty />}
      {props[name] ?? <Empty />}
    </section>
  );
}

// no-mistakes-disable-next-line react-no-nullish-react-node -- retained probe
export const Hidden = (props: Props) => props.children ?? <Empty />;
