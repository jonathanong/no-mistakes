"""Builder identity and disclosed capacity-only measurement hosts."""
def stable_machine(machine):
    required = ('uname', 'affinity', 'imageOS', 'imageVersion', 'cpuinfo', 'meminfo')
    assert all(key in machine for key in required)
    uname = machine['uname']
    assert len(uname) == 5
    return {'uname': [uname[0], uname[2], uname[3], uname[4]],
            'affinity': machine['affinity'], 'imageOS': machine['imageOS'],
            'imageVersion': machine['imageVersion'], 'cpuinfo': machine['cpuinfo']}


def verify_measurement(machine):
    stable_machine(machine)
    assert machine['uname'][4] == 'aarch64'
    assert machine['imageOS'] == 'ubuntu24-arm64'
    assert machine['imageVersion'] in ('20261018.154.1', '20261012.149.1')
    assert len(machine['affinity']) == 4
    return machine
