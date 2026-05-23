import type { ReactNode as Node } from "react";
import UI from "react";
import * as R from "react";

type Slot = Node;
type Alias = (Slot);
export type ExportedSlot = R.ReactNode;
type LiteralProps = {
  "footer-slot"?: Alias;
  ignored?: string;
};

export interface ExportedProps {
  aside?: ExportedSlot;
  nav?: UI.ReactNode;
}

interface Props {
  header?: React.ReactNode;
  footer?: Alias;
  title?: string;
}

const typedSlot: Alias = null;
var globalSlot: UI.ReactNode = null;
const props: Props = {};

export const Arrow = ({ header = null }: Props) => header ?? <DefaultHeader />;

export const Literal = ({ "footer-slot": footerSlot }: LiteralProps) =>
  footerSlot ?? <DefaultFooter />;

export const ExportedObject = ({ aside, nav }: ExportedProps) => (
  <>
    {aside ?? <DefaultFooter />}
    {nav ?? <DefaultFooter />}
  </>
);

export const FromObject = () => props.footer ?? <DefaultFooter />;

export const FromDestructure = () => {
  const { footer = null } = props;
  return footer ?? <DefaultFooter />;
};

export const Direct = () => typedSlot ?? <DefaultFooter />;

export const NamespaceDirect = () => globalSlot ?? <DefaultFooter />;

export function VarScope() {
  if (Math.random()) {
    var blockSlot: R.ReactNode = null;
  }
  return blockSlot ?? <DefaultFooter />;
}

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
