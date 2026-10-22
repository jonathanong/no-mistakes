#!/bin/sh
# Literal global options do not turn read-only identity queries into writes.
git -C . config user.email
git --git-dir=.git config --get user.email || echo missing
'git' "config" --get-all user.name fallback
"git" -c color.ui=false 'config' --get-regexp user.email fallback
git --work-tree '.' config --get-urlmatch user.email fallback
'git' -C . config user.email 2>/dev/null
# 'git' -C . config user.email bot@example.com
echo "'git' -C . config user.email bot@example.com"
printf '%s' 'git --git-dir=.git config user.email bot@example.com'
notgit -C . config user.email bot@example.com
git -C . status user.email bot@example.com
git --version config user.email bot@example.com
git --exec-path config user.email bot@example.com
git -C ; config user.email bot@example.com
# Attached short options are rejected by Git before config can execute.
git -C. config user.email bot@example.com
git -ccolor.ui=false config user.email bot@example.com
git -c'color.ui=false' config user.name Bot
# Inline comments cannot turn missing or read-only arguments into writes.
git -C . config # user.email Bot
git -C . config user.email # Bot
'git' config --get user.email # fallback
git '--git-dir=.git' config --get user.email || echo missing
git "--work-tree=$ROOT" config user.name
