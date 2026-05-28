export function PageWithAside({ children }: { children: React.ReactNode }) {
  return (
    <div className="grid grid-cols-[1fr_320px]">
      <main>{children}</main>
      <aside />
    </div>
  )
}
