struct Helper;

impl Helper {
    #[cfg(test)]
    fn impl_only() {}
}

trait Contract {
    #[cfg(test)]
    fn trait_only();
}

extern "C" {
    #[cfg(test)]
    fn foreign_only();
}
