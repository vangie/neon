//! Wrapper around `std::env::var` for parsing environment variables.

use std::{fmt::Display, str::FromStr};
use serde::{Deserialize, Serialize};

/// For types `V` that implement [`FromStr`].
pub fn var<V, E>(varname: &str) -> Option<V>
where
    V: FromStr<Err = E>,
    E: Display,
{
    match std::env::var(varname) {
        Ok(s) => Some(
            s.parse()
                .map_err(|e| {
                    format!("failed to parse env var {varname} using FromStr::parse: {e:#}")
                })
                .unwrap(),
        ),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => {
            panic!("env var {varname} is not unicode")
        }
    }
}

/// For types `V` that implement [`serde::de::DeserializeOwned`].
pub fn var_serde_json_string<V>(varname: &str) -> Option<V>
where
    V: serde::de::DeserializeOwned,
{
    match std::env::var(varname) {
        Ok(s) => Some({
            let value = serde_json::Value::String(s);
            serde_json::from_value(value)
                .map_err(|e| {
                    format!("failed to parse env var {varname} as a serde_json json string: {e:#}")
                })
                .unwrap()
        }),
        Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => {
            panic!("env var {varname} is not unicode")
        }
    }
}

#[cfg(test)]
mod test_env {
    use super::*;
    use std::env;
    use std::os::unix::ffi::OsStringExt;
    use std::ffi::OsString;

    #[test]
    fn test_var_present() {
        env::set_var("TEST_VAR", "42");
        let result: Option<i32> = var("TEST_VAR");
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_var_not_present() {
        env::remove_var("TEST_VAR_MISSING");
        let result: Option<i32> = var("TEST_VAR_MISSING");
        assert_eq!(result, None);
    }

    #[test]
    #[should_panic(expected = "failed to parse env var TEST_VAR_INVALID")]
    fn test_var_invalid_value() {
        env::set_var("TEST_VAR_INVALID", "not_a_number");
        let _: Option<i32> = var("TEST_VAR_INVALID");
    }

    #[test]
    #[should_panic(expected = "env var TEST_VAR_UNICODE is not unicode")]
    fn test_var_non_unicode() {
        let invalid_unicode = vec![0xFF, 0xFF];
        let os_string = OsString::from_vec(invalid_unicode);
        env::set_var("TEST_VAR_UNICODE", os_string);
        let _: Option<i32> = var("TEST_VAR_UNICODE");
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        field: String,
    }

    #[test]
    fn test_var_serde_json_string_present() {
        env::set_var("TEST_JSON", "test value");
        let result: Option<String> = var_serde_json_string("TEST_JSON");
        assert_eq!(result, Some("test value".to_string()));
    }

    #[test]
    fn test_var_serde_json_string_not_present() {
        env::remove_var("TEST_JSON_MISSING");
        let result: Option<String> = var_serde_json_string("TEST_JSON_MISSING");
        assert_eq!(result, None);
    }

    #[test]
    fn test_var_serde_json_string_complex_type() {
        env::set_var("TEST_STRUCT", "test value");
        let result: Option<String> = var_serde_json_string("TEST_STRUCT");
        assert_eq!(result, Some("test value".to_string()));
    }

    #[test]
    #[should_panic(expected = "failed to parse env var TEST_JSON_INVALID")]
    fn test_var_serde_json_string_invalid() {
        env::set_var("TEST_JSON_INVALID", "invalid json");
        let _: Option<TestStruct> = var_serde_json_string("TEST_JSON_INVALID");
    }

    #[test]
    #[should_panic(expected = "env var TEST_JSON_UNICODE is not unicode")]
    fn test_var_serde_json_string_non_unicode() {
        let invalid_unicode = vec![0xFF, 0xFF];
        let os_string = OsString::from_vec(invalid_unicode);
        env::set_var("TEST_JSON_UNICODE", os_string);
        let _: Option<String> = var_serde_json_string("TEST_JSON_UNICODE");
    }
}
