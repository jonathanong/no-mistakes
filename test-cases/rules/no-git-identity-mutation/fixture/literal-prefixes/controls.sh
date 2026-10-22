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
