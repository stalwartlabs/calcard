pub mod icalendar;
pub mod jscalendar;
pub mod jscontact;
pub mod tokenizer;
pub mod vcard;

/*
#[cfg(test)]
#[allow(clippy::too_many_lines)]
mod test {
    use crate::{unfold_lines, ContentLine, Parser};
    use alloc::borrow::Cow;

    #[test]
    fn test_complete_example() {
        let data = [
            "BEGIN:VCALENDAR",
            "VERSION:2.0",
            "PRODID:nl.whynothugo.todoman",
            "BEGIN:VTODO",
            "DTSTAMP:20231126T095923Z",
            "DUE;TZID=Asia/Shanghai:20231128T090000",
            "SUMMARY:dummy todo for parser tests",
            "UID:565f48cb5b424815a2ba5e56555e2832@destiny.whynothugo.nl",
            "END:VTODO",
            "END:VCALENDAR",
            // Note: this calendar is not entirely semantically valid;
            // it is missing the timezone which is referred to in DUE.
        ]
        .join("\r\n");

        let mut parser = Parser::new(&data);
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "BEGIN:VCALENDAR",
                name: "BEGIN",
                params: "",
                value: "VCALENDAR"
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "VERSION:2.0",
                name: "VERSION",
                params: "",
                value: "2.0",
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "PRODID:nl.whynothugo.todoman",
                name: "PRODID",
                params: "",
                value: "nl.whynothugo.todoman",
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "BEGIN:VTODO",
                name: "BEGIN",
                params: "",
                value: "VTODO",
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "DTSTAMP:20231126T095923Z",
                name: "DTSTAMP",
                params: "",
                value: "20231126T095923Z",
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "DUE;TZID=Asia/Shanghai:20231128T090000",
                name: "DUE",
                params: "TZID=Asia/Shanghai",
                value: "20231128T090000",
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "SUMMARY:dummy todo for parser tests",
                name: "SUMMARY",
                params: "",
                value: "dummy todo for parser tests",
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "UID:565f48cb5b424815a2ba5e56555e2832@destiny.whynothugo.nl",
                name: "UID",
                params: "",
                value: "565f48cb5b424815a2ba5e56555e2832@destiny.whynothugo.nl",
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "END:VTODO",
                name: "END",
                params: "",
                value: "VTODO",
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "END:VCALENDAR",
                name: "END",
                params: "",
                value: "VCALENDAR",
            })
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_empty_data() {
        let data = "";
        let mut parser = Parser::new(data);
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_empty_lines() {
        // A line followed by CRLF is a different code-path than a line followed by EOF.
        let data = "\r\n";
        let mut parser = Parser::new(data);
        let line = parser.next().unwrap();
        assert_eq!(
            line,
            ContentLine {
                raw: "",
                name: "",
                params: "",
                value: "",
            }
        );
        assert_eq!(line.raw(), "");
        assert_eq!(line.name(), "");
        assert_eq!(line.params(), "");
        assert_eq!(line.value(), "");
        // FIXME: trailing empty lines are swallowed.
        // assert_eq!(
        //     parser.next(),
        //     Some(ContentLine {
        //         raw: "",
        //         name: "",
        //         params: "",
        //         value: "",
        //     })
        // );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_line_with_params() {
        // A line with ending in CRLF is a different code-path than a line in EOF.
        let data = [
            "DTSTART;TZID=America/New_York:19970902T090000",
            "DTSTART;TZID=America/New_York:19970902T090000",
        ]
        .join("\r\n");
        let mut parser = Parser::new(&data);
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "DTSTART;TZID=America/New_York:19970902T090000",
                name: "DTSTART",
                params: "TZID=America/New_York",
                value: "19970902T090000",
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "DTSTART;TZID=America/New_York:19970902T090000",
                name: "DTSTART",
                params: "TZID=America/New_York",
                value: "19970902T090000",
            })
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_line_with_dquote() {
        // A line with ending in CRLF is a different code-path than a line in EOF.
        let data = [
            "SUMMARY:This has \"some quotes\"",
            "DTSTART;TZID=\"local;VALUE=DATE-TIME\":20150304T184500",
        ]
        .join("\r\n");
        let mut parser = Parser::new(&data);
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "SUMMARY:This has \"some quotes\"",
                name: "SUMMARY",
                params: "",
                value: "This has \"some quotes\"",
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "DTSTART;TZID=\"local;VALUE=DATE-TIME\":20150304T184500",
                name: "DTSTART",
                params: "TZID=\"local;VALUE=DATE-TIME\"",
                value: "20150304T184500",
            })
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_continuation_line() {
        // A line with ending in CRLF is a different code-path than a line in EOF.
        let data = [
            "X-JMAP-LOCATION;VALUE=TEXT;X-JMAP-GEO=\"geo:52.123456,4.123456\";",
            " X-JMAP-ID=03453afa-71fc-4893-ba70-a7436bb6d56c:Name of place",
            "X-JMAP-LOCATION;VALUE=TEXT;X-JMAP-GEO=\"geo:52.123456,4.123456\";",
            " X-JMAP-ID=03453afa-71fc-4893-ba70-a7436bb6d56c:Name of place",
        ]
        .join("\r\n");
        let mut parser = Parser::new(&data);
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: &[
                    "X-JMAP-LOCATION;VALUE=TEXT;X-JMAP-GEO=\"geo:52.123456,4.123456\";",
                    " X-JMAP-ID=03453afa-71fc-4893-ba70-a7436bb6d56c:Name of place",
                ]
                .join("\r\n"),
                name: "X-JMAP-LOCATION",
                params: "VALUE=TEXT;X-JMAP-GEO=\"geo:52.123456,4.123456\";\r\n X-JMAP-ID=03453afa-71fc-4893-ba70-a7436bb6d56c",
                value: "Name of place",
            })
        );
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: &[
                    "X-JMAP-LOCATION;VALUE=TEXT;X-JMAP-GEO=\"geo:52.123456,4.123456\";",
                    " X-JMAP-ID=03453afa-71fc-4893-ba70-a7436bb6d56c:Name of place",
                ]
                .join("\r\n"),
                name: "X-JMAP-LOCATION",
                params: "VALUE=TEXT;X-JMAP-GEO=\"geo:52.123456,4.123456\";\r\n X-JMAP-ID=03453afa-71fc-4893-ba70-a7436bb6d56c",
                value: "Name of place",
            })
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_invalid_lone_name() {
        let data = "BEGIN";
        let mut parser = Parser::new(data);
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "BEGIN",
                name: "BEGIN",
                params: "",
                value: "",
            })
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_invalid_name_with_params() {
        let data = "DTSTART;TZID=America/New_York";
        let mut parser = Parser::new(data);
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "DTSTART;TZID=America/New_York",
                name: "DTSTART",
                params: "TZID=America/New_York",
                value: "",
            })
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_invalid_name_with_trailing_semicolon() {
        let data = "DTSTART;";
        let mut parser = Parser::new(data);
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "DTSTART;",
                name: "DTSTART",
                params: "",
                value: "",
            })
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_invalid_name_with_trailing_colon() {
        let data = "DTSTART:";
        let mut parser = Parser::new(data);
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "DTSTART:",
                name: "DTSTART",
                params: "",
                value: "",
            })
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_remainder() {
        let data = ["BEGIN:VTODO", "SUMMARY:Do the thing"].join("\r\n");
        let mut parser = Parser::new(&data);
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "BEGIN:VTODO",
                name: "BEGIN",
                params: "",
                value: "VTODO",
            })
        );
        assert_eq!(parser.remainder(), "SUMMARY:Do the thing");
        assert_eq!(
            parser.next(),
            Some(ContentLine {
                raw: "SUMMARY:Do the thing",
                name: "SUMMARY",
                params: "",
                value: "Do the thing",
            })
        );
        assert_eq!(parser.next(), None);
    }

    #[test]
    fn test_fold_multiline() {
        assert_eq!(
            unfold_lines("UID:horrible-\r\n example"),
            "UID:horrible-example"
        );
        assert_eq!(unfold_lines("UID:X\r\n Y"), "UID:XY");
        assert_eq!(unfold_lines("UID:X\r\n "), "UID:X");
        assert_eq!(
            unfold_lines("UID:quite\r\n a\r\n few\r\n lines"),
            "UID:quiteafewlines"
        );
    }

    #[test]
    #[should_panic(expected = "continuation line is not a continuation line")]
    fn test_fold_multiline_missing_whitespace() {
        unfold_lines("UID:two\r\nlines");
    }

    #[test]
    fn test_normalise_folds_short() {
        let data = "SUMMARY:Hello there";
        let mut parser = Parser::new(data);
        let line = parser.next().unwrap();
        assert_eq!(parser.next(), None);

        assert_eq!(line.normalise_folds(), data);
        assert!(matches!(line.normalise_folds(), Cow::Borrowed(_)));
    }

    #[test]
    fn test_normalise_folds_with_carrige_returns() {
        let data = "SUMMARY:Hello \rthere";
        let mut parser = Parser::new(data);
        let line = parser.next().unwrap();
        assert_eq!(parser.next(), None);

        assert_eq!(line.normalise_folds(), data);
        assert!(matches!(line.normalise_folds(), Cow::Borrowed(_)));
    }

    #[test]
    fn test_normalise_folds_with_newlines() {
        let data = "SUMMARY:Hello \nthere";
        let mut parser = Parser::new(data);
        let line = parser.next().unwrap();
        assert_eq!(parser.next(), None);

        assert_eq!(line.normalise_folds(), data);
        assert!(matches!(line.normalise_folds(), Cow::Borrowed(_)));
    }

    #[test]
    fn test_normalise_folds_too_many_folds() {
        let data = "SUMMARY:Hello \r\n \r\n there";
        let mut parser = Parser::new(data);
        let line = parser.next().unwrap();
        assert_eq!(parser.next(), None);

        let expected = "SUMMARY:Hello there";
        assert_eq!(line.normalise_folds(), expected);
    }

    #[test]
    fn test_normalise_folds_long() {
        let data = [
            "SUMMARY:Some really long text that nobody ",
            " cares about, but is wrapped in two lines.",
        ]
        .join("\r\n");
        let mut parser = Parser::new(&data);
        let line = parser.next().unwrap();
        assert_eq!(parser.next(), None);

        let expected = [
            "SUMMARY:Some really long text that nobody cares about, but is wrapped in t",
            " wo lines.",
        ]
        .join("\r\n");
        assert_eq!(line.normalise_folds(), expected);
    }

    #[test]
    fn test_normalise_folds_multibyte() {
        // This is 59 characters, 161 octets
        let data = "SUMMARY:動千首看院波未假遠子到花，白六到星害，馬吃牠說衣欠去皮香收司意，青個話化汁喜視娘以男雪青土已升斤法兌。";
        let mut parser = Parser::new(data);
        let line = parser.next().unwrap();
        assert_eq!(parser.next(), None);

        let expected = [
            // Keep in mind that CR counts, but LF does not.
            "SUMMARY:動千首看院波未假遠子到花，白六到星害，馬吃牠", // 74
            " 說衣欠去皮香收司意，青個話化汁喜視娘以男雪青土已",    // 73
            " 升斤法兌。",                                          // 16
        ]
        .join("\r\n");
        assert_eq!(line.normalise_folds(), expected);
    }

    #[test]
    fn test_normalise_folds_multibyte_noop() {
        // This is 59 characters, 161 octets
        let data = [
            // Keep in mind that CR counts, but LF does not.
            "SUMMARY:動千首看院波未假遠子到花，白六到星害，馬吃牠", // 74
            " 說衣欠去皮香收司意，青個話化汁喜視娘以男雪青土已",    // 73
            " 升斤法兌。",                                          // 16
        ]
        .join("\r\n");
        let mut parser = Parser::new(&data);
        let line = parser.next().unwrap();
        assert_eq!(parser.next(), None);

        assert_eq!(line.normalise_folds(), data);
        assert!(matches!(line.normalise_folds(), Cow::Borrowed(_)));
    }

    #[test]
    fn test_unfold_params_with_trailing_crlf() {
        let data = ";\r\n";
        let mut parser = Parser::new(data);
        let line = parser.next().unwrap();
        assert_eq!(line.raw(), ";");
        assert_eq!(line.name(), "");
        assert_eq!(line.params(), "");
        assert_eq!(line.value(), "");
    }

    #[test]
    fn test_unfold_name_with_trailing_crlf() {
        let data = "\r\n";
        let mut parser = Parser::new(data);
        let line = parser.next().unwrap();
        assert_eq!(line.raw(), "");
        assert_eq!(line.name(), "");
        assert_eq!(line.params(), "");
        assert_eq!(line.value(), "");
    }

    #[test]
    fn test_unfold_value_with_trailing_crlf() {
        let data = ";:\r\n";
        let mut parser = Parser::new(data);
        let line = parser.next().unwrap();
        assert_eq!(line.raw(), ";:");
        assert_eq!(line.name(), "");
        assert_eq!(line.params(), "");
        assert_eq!(line.value(), "");
    }
}
*/
