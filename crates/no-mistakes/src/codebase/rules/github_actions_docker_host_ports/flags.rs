pub(super) fn short_kind(ch: char) -> Option<bool> {
    match ch {
        'd' | 'i' | 'q' | 't' | 'P' => Some(false),
        'a' | 'c' | 'e' | 'h' | 'l' | 'm' | 'p' | 'u' | 'v' | 'w' => Some(true),
        _ => None,
    }
}

pub(super) fn long_arg(name: &str) -> Option<bool> {
    const BOOLS: &[&str] = &[
        "detach",
        "help",
        "init",
        "interactive",
        "no-healthcheck",
        "privileged",
        "publish-all",
        "read-only",
        "rm",
        "sig-proxy",
        "tty",
        "oom-kill-disable",
        "quiet",
        "disable-content-trust",
        "use-api-socket",
        "tls",
        "tlsverify",
    ];
    const ARGS: &[&str] = &[
        "add-host",
        "attach",
        "cgroupns",
        "cidfile",
        "cpu-shares",
        "cpus",
        "entrypoint",
        "env",
        "env-file",
        "expose",
        "gpus",
        "group-add",
        "health-cmd",
        "health-interval",
        "health-retries",
        "health-start-period",
        "health-timeout",
        "hostname",
        "ip",
        "ipc",
        "label",
        "label-file",
        "link",
        "log-driver",
        "log-opt",
        "memory",
        "memory-swap",
        "mount",
        "name",
        "net",
        "network",
        "pid",
        "platform",
        "publish",
        "pull",
        "restart",
        "runtime",
        "security-opt",
        "shm-size",
        "stop-signal",
        "sysctl",
        "tmpfs",
        "ulimit",
        "user",
        "volume",
        "volumes-from",
        "workdir",
        "cap-add",
        "cap-drop",
        "device",
        "device-cgroup-rule",
        "dns",
        "dns-option",
        "dns-search",
        "domainname",
        "memory-reservation",
        "stop-timeout",
        "health-start-interval",
        "userns",
        "uts",
        "network-alias",
        "cpuset-cpus",
        "cpuset-mems",
        "cpu-period",
        "cpu-quota",
        "blkio-weight",
        "isolation",
        "mac-address",
        "ip6",
        "storage-opt",
        "annotation",
        "oom-score-adj",
        "pids-limit",
        "cgroup-parent",
        "kernel-memory",
        "link-local-ip",
        "memory-swappiness",
        "context",
        "host",
        "config",
        "log-level",
        "tlscacert",
        "tlscert",
        "tlskey",
    ];
    if BOOLS.contains(&name) {
        Some(false)
    } else if ARGS.contains(&name) {
        Some(true)
    } else {
        None
    }
}

pub(super) fn is_assign(word: &str) -> bool {
    let Some((name, _)) = word.split_once('=') else {
        return false;
    };
    let mut chars = name.chars();
    match chars.next() {
        Some(ch) if ch.is_ascii_alphabetic() || ch == '_' => {
            chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        }
        _ => false,
    }
}

pub(super) fn basename(word: &str) -> &str {
    word.rsplit(['/', '\\']).next().unwrap_or(word)
}
