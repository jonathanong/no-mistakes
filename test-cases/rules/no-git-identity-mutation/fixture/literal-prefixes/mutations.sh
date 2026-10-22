#!/bin/sh
# These are static analyzer inputs only; never execute this fixture.
git config user.email bot@example.com
git -C . config user.email bot@example.com
git -C 'directory with spaces' config user.name 'Build Bot'
git -C . -c color.ui=false --no-pager config user.email bot@example.com
git --git-dir=.git config --local user.email bot@example.com
git --git-dir '.git' --work-tree . config user.name Bot
git --namespace=automation --bare config user.email bot@example.com
git --config-env=core.editor=EDITOR config user.name Bot
git --exec-path=/usr/bin --attr-source=HEAD config user.email bot@example.com
'git' config user.email bot@example.com
"git" config user.name Bot
git 'config' user.email bot@example.com
'git' "config" user.name Bot
"git" '-C' '.' 'config' user.email bot@example.com
git -c 'color.ui=false' config user.name Bot
git -C '--get' config user.email bot@example.com
git -C . \
  'config' \
  user.email bot@example.com
git -C "$ROOT" config user.email bot@example.com
git --git-dir="`directory`" config user.name Bot
# A preceding read must never swallow a following identity write.
git -C . config --get user.email; git config user.email bot@example.com
git -C . config --get user.email && git config user.email bot@example.com
git -C . config --get user.email || git config user.email bot@example.com
git -C . config --file 'path;name' user.email bot@example.com
git -C . config --file="path|name" user.email bot@example.com
# Existing plain-command suffix forms retain their findings.
git config --file $CONFIG user.email Bot
git config --file $(config_path) user.email Bot
git config --file `config_path` user.email Bot
# Quotes around the complete equals-form option are removed by the shell.
git '--git-dir=.git' config user.email bot@example.com
git "--work-tree=$ROOT" config user.name Bot
git -C . config --file path#name user.email Bot
git -C . config --file 'path#name' user.email Bot
git -C . config --file "#path" user.email Bot
