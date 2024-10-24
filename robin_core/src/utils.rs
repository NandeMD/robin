use pbr::ProgressBar;

pub const INT_FLOAT_REGEX: &str = r"[-+]?(?:\d*\.*\d+)";

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn create_progress_bar(total: u64, msg: &str) -> ProgressBar<std::io::Stdout> {
    let mut pb = ProgressBar::new(total);

    pb.message(msg);
    pb.format("╢▌▌░╟");

    pb.set_max_refresh_rate(Some(std::time::Duration::from_millis(100)));
    pb.show_bar = true;
    pb.show_speed = false;
    pb.show_counter = true;
    pb.show_time_left = false;
    pb.show_percent = true;
    pb.show_message = true;

    pb
}