macro_rules! ewrap {
    (err $e:ident $(,$context:expr)* $(=> $($arg:expr),*)*) => {{
        let mut msg = "at ".to_string();
        msg.push_str(&format!("at {}:{}", file!(), line!()));
        $(
            $(
                let arg = format!("\nwith args - {}: {:#?}", stringify!($arg), $arg);
                msg.push_str(&arg);
            )*
        )*
        let err = e.context(msg);
        $(
            let err = err.context($context);
        )*
        err.into()
    }};

    (catch $($context:expr),* $(=> $($arg:expr),*)*) => {{
        move |e| {
            let kind = e.kind().into();
            ewrap!(err e $(,$context)*, kind $(=> $($arg),*)*)
        }
    }};

    ($($context:expr),* $(=> $($arg:expr),*)*) => {{
        move |e| {
            ewrap!(err e $(,$context)* $(=> $($arg),*)*)
        }
    }};
}
