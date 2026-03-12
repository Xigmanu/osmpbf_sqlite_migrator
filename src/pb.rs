use std::fmt::Write;

use indicatif::{ProgressBar, ProgressState, ProgressStyle, style::TemplateError};

pub fn make_pb(len: u64) -> Result<ProgressBar, TemplateError> {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:.cyan/blue}] {pos}/{len:5} | {msg} ({eta})",
        )?
        .with_key("eta", |state: &ProgressState, w: &mut dyn Write| {
            write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
        })
        .progress_chars("#-"),
    );

    Ok(pb)
}
