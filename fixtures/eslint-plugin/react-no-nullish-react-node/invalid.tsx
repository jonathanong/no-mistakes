import type { ReactNode } from "react";

type Slot = ReactNode;

interface Props {
  footer?: React.ReactNode;
  header?: Slot;
}

export function Layout(props: Props) {
  const { header } = props;
  return (
    <section>
      {props.footer ?? <DefaultFooter />}
      {header ?? <DefaultHeader />}
    </section>
  );
}

function DefaultFooter() {
  return null;
}

function DefaultHeader() {
  return null;
}
