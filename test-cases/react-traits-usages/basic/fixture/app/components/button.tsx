// Component under test plus its exported prop types. Used by `react usages`
// fixtures: ButtonProps (interface) + ButtonVariant (type alias) are the prop
// type names the query should surface.
export interface ButtonProps {
  variant: string;
  onClick: () => void;
}

export type ButtonVariant = "primary" | "secondary";

export function Button(props: ButtonProps) {
  return <button onClick={props.onClick}>{props.variant}</button>;
}

export default Button;
