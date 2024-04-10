/// Builds pattern lazily.
#[macro_export]
macro_rules! regex_builder {
    ( $builder:expr ) => {{
        use regex::Regex;
        use std::sync::LazyLock;

        static EXPR: LazyLock<Regex> =
            LazyLock::new(|| $builder.build().expect("Regex failed to compile"));
        &EXPR
    }};
}

/// Lazy regex.
/// Only compiles once and prevents an extra variable.
#[macro_export]
macro_rules! regex {
    ( $regex:expr ) => {{
        use $crate::regex_builder;

        regex_builder!({
            use regex::RegexBuilder;
            RegexBuilder::new($regex)
        })
    }};
}

/// Try blocks with a specified error type
/// Prevents having to assign to a variable
#[macro_export]
macro_rules! try_expect {
    ( $error_type:ty, $expect_message:expr, $code_block:expr ) => {{
        let err: $error_type = try { $code_block };
        err.expect($expect_message)
    }};
}
