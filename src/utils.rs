use failure::Fail;
use regex;

fn format_error<E: Fail>(error: E) -> String {
    let mut result = String::new();
    let mut chain: Vec<&Fail> = Vec::new();
    let mut iter: Option<&Fail> = Some(&error);
    while let Some(e) = iter {
        chain.push(e);
        iter = e.cause();
    }
    for err in chain.into_iter().rev() {
        result.push_str(&format!("{}\n", err));
    }
    if let Some(bt) = error.backtrace() {
        let regexp = regex::Regex::new("payments_lib").unwrap();
        let bt = format!("{}", bt);
        let lines: Vec<&str> = bt.split("\n").skip(1).collect();
        if lines.len() > 0 {
            result.push_str("\nRelevant backtrace:\n");
        }
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
