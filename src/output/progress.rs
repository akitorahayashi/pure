use indicatif::ProgressStyle;

pub fn discovery_spinner_style() -> ProgressStyle {
    ProgressStyle::with_template("{spinner:.cyan} {msg}").unwrap().tick_chars("|/-\\")
}

pub fn size_progress_style() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>6}/{len:>6}")
        .unwrap()
        .progress_chars("=|-")
}

pub fn deletion_progress_style() -> ProgressStyle {
    ProgressStyle::with_template("[{elapsed_precise}] {bar:40.cyan/blue} {pos:>6}/{len:>6}")
        .unwrap()
        .progress_chars("=|-")
}
