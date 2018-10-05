macro_rules! ectx {
    (try err $e:expr $(,$context:expr)* $(=> $($arg:expr),*)*) => {{
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
        err
    }};

    (err $e:expr $(,$context:expr)* $(=> $($arg:expr),*)*) => {{
        let err = ectx!(try err $e $(,$context)* $(=> $($arg),*)*);
        err.into()
    }};

    (try convert err $e:expr $(,$context:expr)* $(=> $($arg:expr),*)*) => {{
        let e = $e.kind().into();
        ectx!(try err $e $(,$context)*, e $(=> $($arg),*)*)
    }};

    (convert err $e:expr $(,$context:expr)* $(=> $($arg:expr),*)*) => {{
        let e = $e.kind().into();
        ectx!(err $e $(,$context)*, e $(=> $($arg),*)*)
    }};

    (try convert $($context:expr),* $(=> $($arg:expr),*)*) => {{
        move |e| {
            ectx!(try convert err e $(,$context)* $(=> $($arg),*)*)
        }
    }};

    (convert $($context:expr),* $(=> $($arg:expr),*)*) => {{
        move |e| {
            ectx!(convert err e $(,$context)* $(=> $($arg),*)*)
        }
    }};

    (try $($context:expr),* $(=> $($arg:expr),*)*) => {{
        move |e| {
            ectx!(try err e $(,$context)* $(=> $($arg),*)*)
        }
    }};

    ($($context:expr),* $(=> $($arg:expr),*)*) => {{
        move |e| {
            ectx!(err e $(,$context)* $(=> $($arg),*)*)
        }
    }};
}
