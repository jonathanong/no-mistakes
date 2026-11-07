const statements = await import('./producer')
statements.fragment.text = process.env.SQL
