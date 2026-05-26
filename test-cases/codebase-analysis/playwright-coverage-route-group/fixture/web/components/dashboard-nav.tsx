const tabs = [
  { key: 'account', label: 'Account', href: '/account/settings' },
  { key: 'profile', label: 'Profile', href: '/profile' },
]

export function DashboardNav() {
  return (
    <nav data-pw="dashboard-nav">
      {tabs.map(tab => (
        <a
          key={tab.key}
          href={tab.href}
          data-pw={`nav-tab-${tab.key}`}
        >
          {tab.label}
        </a>
      ))}
    </nav>
  )
}
