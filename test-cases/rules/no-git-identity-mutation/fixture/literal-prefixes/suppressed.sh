#!/bin/sh
# no-mistakes-disable-next-line no-git-identity-mutation
'git' -C . config user.email bot@example.com
"git" config user.name Bot # no-mistakes-disable-line no-git-identity-mutation
