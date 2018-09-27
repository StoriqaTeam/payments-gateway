macro_rules! ectx {
    (raw $e:ident, $context:expr, $kind:expr) => {
        ectx!(raw $e, $context, $kind, )
    };

    ($context:expr, $kind:expr) => {
        ectx!($context, $kind, )
    };

    (raw $e:ident, $context:expr, $kind:expr, $($arg:expr),*) => {{
        let mut msg = format!("at {}:{}", file!(), line!());
        $(
            let arg = format!(" {}: {:#?}", stringify!($arg), $arg);
            msg.push_str(&arg);
        )*
        $e.context(msg).context($context).context($kind).into()
    }};

    (catch $context:expr, $($arg:expr),*) => {{
        move |e| {
            let kind = e.kind().into();
            ectx!(raw e, $context, kind, $($arg),*)
        }
    }};


    ($context:expr, $kind:expr, $($arg:expr),*) => {{
        move |e| {
            ectx!(raw e, $context, $kind, $($arg),*)
        }
    }};

}
