use std::borrow::Cow;

use doclog::Log;

#[test]
pub fn make_owned() {
    let log = {
        let title = "This is a test".to_string();

        let mut log = Log::info().title(title.as_str(), true, true);
        log = log.document(Cow::Borrowed("this is a test🥹\nthis is a test"), |doc| {
            doc.highlight_section(0..4, None)
                .highlight_cursor(8, None)
                .highlight_section(10..20, None)
        });
        log = log.document(Cow::Borrowed("this is a test🥹\nthis is a test"), |doc| {
            doc.highlight_section_message(0..4, Cow::Borrowed("www"), None)
                .highlight_cursor_message(8, Cow::Borrowed("xyz"), None)
                .highlight_section_message(10..20, Cow::Borrowed("www"), None)
        });

        log.make_owned()
    };

    log.log_ansi_text();
}
