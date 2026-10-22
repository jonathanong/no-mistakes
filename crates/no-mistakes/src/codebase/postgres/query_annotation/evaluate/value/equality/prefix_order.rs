// Both builds execute this ordering; selected hooks instrument only SQL equality.
macro_rules! compare {
    ($a:expr, $b:expr, $c:expr, $a2:expr, $b2:expr, $c2:expr) => {
        $b == $b2 && $c == $c2 && hooks::sql_component!($a, $a2)
    };
}
pub(super) use compare;
