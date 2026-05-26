import { LayoutButton } from "./components/layout-button";

export default function Layout({ children }: { children: React.ReactNode }) {
  return (
    <>
      <LayoutButton />
      {children}
    </>
  );
}
