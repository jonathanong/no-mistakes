import type { ReactNode as Node } from "react";

type Slot = Node;
type Alias = (Slot);
type LiteralProps = {
  "footer-slot"?: Alias;
  ignored?: string;
};

interface Props {
  header?: React.ReactNode;
  footer?: Alias;
  title?: string;
}

const typedSlot: Alias = null;
const props: Props = {};

export const Arrow = ({ header = null }: Props) => header ?? <DefaultHeader />;

export const Literal = ({ "footer-slot": footerSlot }: LiteralProps) =>
  footerSlot ?? <DefaultFooter />;

export const FromObject = () => props.footer ?? <DefaultFooter />;

export const FromDestructure = () => {
  const { footer = null } = props;
  return footer ?? <DefaultFooter />;
};

export const Direct = () => typedSlot ?? <DefaultFooter />;

export const Ignored = ({ title }: Props) => {
  const computed = props["footer"];
  const unknown = {};
  return (
    <>
      {title ?? "title"}
      {computed ?? <DefaultFooter />}
      {unknown.footer ?? <DefaultFooter />}
      {`${title}` ?? "title"}
    </>
  );
};

const FunctionExpression = function (slot: Alias) {
  return slot ?? <DefaultFooter />;
};

function DefaultHeader() {
  return null;
}

function DefaultFooter() {
  return null;
}

FunctionExpression;
