macro_rules! ewrap {
    (raw $e:ident, $source:expr, $kind:expr $(,$arg:expr)*) => {{
        let mut msg = "at ".to_string();
        msg.push_str(&format!("{}:{}", file!(), line!()));
        $(
            let arg = format!("\nargs - {}: {:#?}", stringify!($arg), $arg);
            msg.push_str(&arg);
        )*
        $e.context($source).context(msg).context($kind).into()
    }};

    (catch $source:expr $(,$arg:expr)*) => {{
        move |e| {
            let kind = e.kind().into();
            ewrap!(raw e, $source, kind $(,$arg)*)
        }
    }};

    ($source:expr, $kind:expr $(,$arg:expr)*) => {{
        move |e| {
            ewrap!(raw e, $source, $kind $(,$arg)*)
        }
    }};
}
