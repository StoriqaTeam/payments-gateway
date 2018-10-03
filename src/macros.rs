macro_rules! ectx {
    (err $e:expr $(,$context:expr)* $(=> $($arg:expr),*)*) => {{
        let mut msg = "at ".to_string();
        msg.push_str(&format!("{}:{}", file!(), line!()));
        $(
            $(
                let arg = format!("\nwith args - {}: {:#?}", stringify!($arg), $arg);
                msg.push_str(&arg);
            )*
        )*
        let err = $e.context(msg);
        $(
            let err = err.context($context);
        )*
        err.into()
    }};

    (catch err $e:expr $(,$context:expr)* $(=> $($arg:expr),*)*) => {{
        let e = $e.kind().into();
        ectx!(err $e $(,$context)*, e $(=> $($arg),*)*)
    }};


    (catch $($context:expr),* $(=> $($arg:expr),*)*) => {{
        move |e| {
            ectx!(catch err e $(,$context)* $(=> $($arg),*)*)
        }
    }};

    ($($context:expr),* $(=> $($arg:expr),*)*) => {{
        move |e| {
            ectx!(err e $(,$context)* $(=> $($arg),*)*)
        }
    }};
}

/// Macro for creating validation errors. It uses json-like syntax
/// `{<field_name>: [<error_code> => <error_message>]`. Field is coming
/// from struct like
///
/// ```
///   struct Form {
///     email: String,
///     password: String
///   }
/// ```
/// In this case email and password are `field_names`.
///
/// `error_code` is smth like "too long", "not an email", etc -
/// i.e. the type of validator that fails. Always
/// use `validator::Validator` enum for that, unless it really doesn't fit.
/// `error_message` is a custom error message.
///
/// # Examples
///
/// ```
/// #[macro_use] extern crate users_lib;
/// extern crate validator;
///
/// use validator::Validator;
///
/// fn main() {
///     let errors = validation_errors!({
///         "email": [Validator::Email.code() => "Invalid email", "exists" => "Already exists"],
///         "password": ["match" => "Doesn't match"]
///     });
/// }
/// ```
macro_rules! validation_errors {
    ({$($field:tt: [$($code:expr => $value:expr),+]),*}) => {{
        use validator;
        use std::borrow::Cow;
        use std::collections::HashMap;

        let mut errors = validator::ValidationErrors::new();
        $(
            $(
                let error = validator::ValidationError {
                    code: Cow::from($code),
                    message: Some(Cow::from($value)),
                    params: HashMap::new(),
                };

                errors.add($field, error);
            )+
        )*

        errors
    }}
}
