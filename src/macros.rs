macro_rules! ewrap {
    (raw $e:ident, $source:expr, $kind:expr) => {{
        let msg = format!("at {}:{}", file!(), line!());
        $e.context($source).context(msg).context($kind).into()
    }};

    (raw $e:ident, $source:expr, $kind:expr, $($arg:expr),*) => {{
        let mut msg = format!("at {}:{}", file!(), line!());
        $(
            let arg = format!("\nargs - {}: {:#?}", stringify!($arg), $arg);
            msg.push_str(&arg);
        )*
        $e.context($source).context(msg).context($kind).into()
    }};

    (catch $source:expr, $($arg:expr),*) => {{
        move |e| {
            let kind = e.kind().into();
            ewrap!(raw e, $source, kind, $($arg),*)
        }
    }};

    ($source:expr, $kind:expr) => {
        move |e| {
            ewrap!(raw e, $source, $kind)
        }
    };

    ($source:expr, $kind:expr, $($arg:expr),*) => {{
        move |e| {
            ewrap!(raw e, $source, $kind, $($arg),*)
        }
    }};

}
