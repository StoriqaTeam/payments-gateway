macro_rules! error_context {
    ($e:ident, $kind:expr) => {
        error_context!($e, $kind,)
    };

    ($e:ident, $kind:expr, $($arg:expr),*) => {{
        let mut msg = format!("at {}:{}", file!(), line!());
        $(
            let arg = format!(" {}: {:?}", stringify!($arg), $arg);
            msg.push_str(&arg);
        )*
        $e.context(msg).context($kind).into()
    }};
}
