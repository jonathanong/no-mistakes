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

export const InlineProps = ({
  header,
  title,
}: {
  header?: React.ReactNode;
  title?: string;
}) => (
  <>
    {header ?? <DefaultHeader />}
    {title}
  </>
);

function DefaultFooter() {
  return null;
}

function DefaultHeader() {
  return null;
}
