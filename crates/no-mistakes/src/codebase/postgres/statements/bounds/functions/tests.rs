mod projection_arguments;

use super::SET_RETURNING;

#[test]
fn every_catalog_set_returning_function_is_known() {
    let Ok(connection) = std::env::var("NO_MISTAKES_TEST_POSTGRES_URL") else {
        assert!(
            std::env::var_os("CI").is_none(),
            "CI must supply the real PostgreSQL lane"
        );
        return;
    };
    let mut command = std::process::Command::new("psql");
    crate::postgres_catalog::connection_environment(&connection, &mut command).unwrap();
    let output = command.args(["-X", "--no-password", "-At", "-v", "ON_ERROR_STOP=1", "-c",
                "SELECT DISTINCT proname FROM pg_proc WHERE pronamespace = 'pg_catalog'::regnamespace AND proretset ORDER BY proname"])
            .output().unwrap();
    assert!(output.status.success(), "pg_proc query failed");
    let names = String::from_utf8(output.stdout).unwrap();
    let missing: Vec<_> = names
        .lines()
        .filter(|name| !SET_RETURNING.contains(name))
        .collect();
    assert!(
        missing.is_empty(),
        "unclassified set-returning built-ins: {missing:?}"
    );
}
