import { TemplateButton } from "./components/template-button";

export default function Template({ children }: { children: React.ReactNode }) {
  return (
    <>
      <TemplateButton />
      {children}
    </>
  );
}
