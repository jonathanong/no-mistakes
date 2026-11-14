use super::super::TempFactoryBinding;
use super::{named_result, TempReturn};

#[test]
fn module_binding_is_not_a_named_factory_result() {
    assert_eq!(
        named_result(&TempFactoryBinding::Named {
            sync: true,
            result: TempReturn::Directory,
        }),
        Some(TempReturn::Directory)
    );
    assert_eq!(
        named_result(&TempFactoryBinding::Module {
            sync: true,
            promises_safe: true,
        }),
        None
    );
}
