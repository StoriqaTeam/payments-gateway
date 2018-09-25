use failure::Fail;
use regex;

fn format_error<E: Fail>(error: E) -> String {
    let mut result = String::new();
    let mut iter: &Fail = &error;
    while let Some(e) = iter.cause() {
        result.push_str(&format!("{}\n", iter));
        iter = e;
    }
    result.push_str(&format!("{}", iter));
    if let Some(bt) = error.backtrace() {
        let regexp = regex::Regex::new("payments_lib").unwrap();
        result.push_str("\nRelevant backtrace: \n");
        let bt = format!("{}", bt);
        let lines: Vec<&str> = bt.split("\n").skip(1).collect();
        lines.chunks(2).for_each(|chunk| {
            if let Some(line1) = chunk.get(0) {
                if regexp.is_match(line1) {
                    result.push_str(line1);
                    result.push_str("\n");
                    if let Some(line2) = chunk.get(1) {
                        result.push_str(line2);
                        result.push_str("\n");
                    }
                }
            }
        });
    }
    result
}

pub fn log_error<E: Fail>(error: E) {
    error!("\n{}", format_error(error));
}
