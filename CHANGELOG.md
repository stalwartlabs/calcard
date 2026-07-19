calcard 0.3.7
================================
- Fix: JSCalendar `rscale` not converted to the iCalendar `RSCALE` rule part.
- Fix: `mailto:` scheme not stripped from calendar addresses when uppercase or mixed case.

calcard 0.3.6
================================
- Fix: `FN` property not generated for vCard when `N` property is present (#22).

calcard 0.3.5
================================
- Include 'ENCODING=' when exporting vCard v3.0 and below.

calcard 0.3.4
================================
- Include `CHARSET=UTF-8` when exporting vCard v3.0 and below.

calcard 0.3.3
================================
- Support `STATUS:CANCELLED` mapping from `VTODO` to JSCalendar (#20).
- Fixed duration parsing for zero duration `PT0S`.

calcard 0.3.2
================================
- Fixed vCard `CELL` to JSContact `mobile` mapping (#15).
- Updated JSCalendar conversion rules according to `draft-ietf-calext-jscalendar-icalendar-21`.

calcard 0.3.1
================================
- Fixed jcal implementation to use lowercase property and component names.
- Updated conversion rules according to the latest JSCalendar-bis specification.

calcard 0.3.0
================================
- JMAP for Calendars support.
- JMAP for Contacts support.

calcard 0.2.0
================================
- JSCalendar parsing and conversion to iCalendar format.
- JSContact parsing and conversion to vCard format.
- Fix: `RRULE` with `UNTIL` dates not parsed correctly (#12).
- Fix: Support multiple periods in `FREEBUSY` component (#4).
- Fix: Incorrect `\ ` conversion (#3).

calcard 0.1.3
================================
- Added some builder methods.

calcard 0.1.1
================================
- Export vCard in legacy formats.

calcard 0.1.0
================================
- Initial release.
