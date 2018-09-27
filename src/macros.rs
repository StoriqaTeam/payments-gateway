macro_rules! error_context {
    ($e:ident, $context:expr, $kind:expr) => {
        error_context!($e, $context, $kind, )
    };

    ($e:ident, $context:expr, $kind:expr, $($arg:expr),*) => {{
        let mut msg = format!("at {}:{}", file!(), line!());
        $(
            let arg = format!(" {}: {:?}", stringify!($arg), $arg);
            msg.push_str(&arg);
        )*
        $e.context(msg).context($context).context($kind).into()
    }};
}
