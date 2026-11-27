use super::*;

#[test]
fn url_credentials_stay_in_child_environment() {
    let mut command = Command::new("psql");
    connection_environment(
        "postgresql://user%2Bname:secret%40value@[::1]:5433/database%20name?sslmode=require",
        &mut command,
    )
    .unwrap();
    assert_eq!(command.get_args().count(), 0);
    let environment = command
        .get_envs()
        .filter_map(|(key, value)| {
            value.map(|value| {
                (
                    key.to_string_lossy().into_owned(),
                    value.to_string_lossy().into_owned(),
                )
            })
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(environment["PGHOST"], "::1");
    assert_eq!(environment["PGUSER"], "user+name");
    assert_eq!(environment["PGPASSWORD"], "secret@value");
    assert_eq!(environment["PGDATABASE"], "database name");
    assert_eq!(environment["PGSSLMODE"], "require");
    assert_eq!(environment["PGCONNECT_TIMEOUT"], "10");
    for invalid in [
        "host=localhost dbname=postgres",
        "https://localhost/db",
        "postgresql://localhost/db#fragment",
        "postgresql://localhost/db?unknown=secret",
        "postgresql://user%FF@localhost/db",
        "postgresql://user:password%FF@localhost/db",
        "postgresql://localhost/db%FF",
    ] {
        assert!(connection_environment(invalid, &mut Command::new("psql")).is_err());
    }
}

#[test]
fn supported_url_parameters_map_to_libpq_environment() {
    let mut command = Command::new("psql");
    connection_environment("postgresql://localhost/db?host=localhost&port=5432&user=reader&dbname=test&sslmode=verify-full&sslrootcert=root.pem&sslcert=client.pem&sslkey=client.key&connect_timeout=5", &mut command).unwrap();
    let environment = command
        .get_envs()
        .filter(|(_, value)| value.is_some())
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(environment.len(), 9);
    assert_eq!(
        environment[std::ffi::OsStr::new("PGCONNECT_TIMEOUT")],
        Some(std::ffi::OsStr::new("5"))
    );
}

#[test]
fn url_can_use_libpq_defaults_for_omitted_components() {
    let mut command = Command::new("psql");
    connection_environment("postgresql:///", &mut command).unwrap();
    assert_eq!(
        command
            .get_envs()
            .filter(|(_, value)| value.is_some())
            .count(),
        1
    );
    assert!(command
        .get_envs()
        .any(|(key, value)| key == "PGSERVICE" && value.is_none()));
}
