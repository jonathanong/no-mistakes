//! Complete catalog generation against a real PostgreSQL cluster
//! (`NO_MISTAKES_TEST_POSTGRES_URL`; CI provides one and never skips these).
mod bounded;
mod current_database;
mod edges;
mod parity;
mod rules;
mod shadowing;
mod support;
