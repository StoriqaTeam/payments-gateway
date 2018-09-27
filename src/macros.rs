macro_rules! ectx {
    ($e:ident, $context:expr, $kind:expr) => {
        ectx!($e, $context, $kind, )
    };

    ($e:ident, $context:expr, $kind:expr, $($arg:expr),*) => {{
        let mut msg = format!("at {}:{}", file!(), line!());
        $(
            let arg = format!(" {}: {:#?}", stringify!($arg), $arg);
            msg.push_str(&arg);
        )*
        $e.context(msg).context($context).context($kind).into()
    }};
}
