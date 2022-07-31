use doclog::Log;

#[test]
pub fn make_owned() {
    let out_log;

    {
        let title = "This is a test".to_string();

        let log = Log::info().title(title.as_str(), true, true);
        out_log = log.make_owned();
    }

    out_log.log_ansi_text();
}
