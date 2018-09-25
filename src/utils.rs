use failure::Fail;

fn format_error<E: Fail>(error: E) -> String {
    let mut result = String::new();
    let mut iter: &Fail = &error;
    while let Some(e) = iter.cause() {
        result.push_str(&format!("{}\n", iter));
        iter = e;
    }
    result.push_str(&format!("{}", iter));
    result
}

pub fn log_error<E: Fail>(error: E) {
    error!("\n{}", format_error(error));
}
