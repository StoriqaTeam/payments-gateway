macro_rules! ewrap {
    (raw $e:ident, $context:expr, $kind:expr) => {
        ewrap!(raw $e, $context, $kind, )
    };

    ($context:expr, $kind:expr) => {
        ewrap!($context, $kind, )
    };

    (raw $e:ident, $context:expr, $kind:expr, $($arg:expr),*) => {{
        let mut msg = format!("at {}:{}", file!(), line!());
        $(
            let arg = format!(" {}: {:#?}", stringify!($arg), $arg);
            msg.push_str(&arg);
        )*
        $e.context($context).context(msg).context($kind).into()
    }};

    (catch $context:expr, $($arg:expr),*) => {{
        move |e| {
            let kind = e.kind().into();
            ewrap!(raw e, $context, kind, $($arg),*)
        }
    }};


    ($context:expr, $kind:expr, $($arg:expr),*) => {{
        move |e| {
            ewrap!(raw e, $context, $kind, $($arg),*)
        }
    }};

}
