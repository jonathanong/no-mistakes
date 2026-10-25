SELECT abs(a.id), abs(b.id) FROM accounts a JOIN accounts b ON a.id = b.id WHERE a.id > 0 ORDER BY a.id FOR KEY SHARE OF a FOR NO KEY UPDATE OF b;
