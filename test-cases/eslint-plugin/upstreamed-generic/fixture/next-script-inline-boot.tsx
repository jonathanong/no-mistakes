const THEME_INIT_SCRIPT = "document.documentElement.dataset.theme = 'dark'";

export default function Layout() {
  return (
    <html>
      <body>
        <script
          id="theme-init"
          nonce={nonce}
          suppressHydrationWarning
          dangerouslySetInnerHTML={{ __html: THEME_INIT_SCRIPT }}
        />
        {children}
      </body>
    </html>
  );
}

