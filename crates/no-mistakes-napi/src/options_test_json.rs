#[cfg(any(test, feature = "test-instrumentation"))]
pub(crate) fn test_json_arg(value: impl TestJsonArg) -> Value {
    value.into_json_value()
}

#[cfg(any(test, feature = "test-instrumentation"))]
pub(crate) trait TestJsonArg {
    fn into_json_value(self) -> Value;
}

#[cfg(any(test, feature = "test-instrumentation"))]
impl TestJsonArg for Value {
    fn into_json_value(self) -> Value {
        self
    }
}

#[cfg(any(test, feature = "test-instrumentation"))]
impl TestJsonArg for &Value {
    fn into_json_value(self) -> Value {
        self.clone()
    }
}

#[cfg(any(test, feature = "test-instrumentation"))]
impl TestJsonArg for String {
    fn into_json_value(self) -> Value {
        serde_json::from_str(&self).unwrap_or_else(|error| panic!("{error}: {self}"))
    }
}

#[cfg(any(test, feature = "test-instrumentation"))]
impl TestJsonArg for &String {
    fn into_json_value(self) -> Value {
        serde_json::from_str(self).unwrap_or_else(|error| panic!("{error}: {self}"))
    }
}

#[cfg(any(test, feature = "test-instrumentation"))]
impl TestJsonArg for &str {
    fn into_json_value(self) -> Value {
        serde_json::from_str(self).unwrap_or_else(|error| panic!("{error}: {self}"))
    }
}
