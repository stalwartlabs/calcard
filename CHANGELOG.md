calcard 0.3.11
================================
- Fix: whitespace immediately preceding a fold was discarded while unfolding, silently joining the words around it (RFC 5545, Section 3.1).
- Fix: `into_jscalendar()` panicked on `VJOURNAL`, `VFREEBUSY`, `VTIMEZONE` and unknown components, none of which have a JSCalendar type.
- Fix: `RRULE` values that could not be parsed were truncated to their first rule part instead of being preserved verbatim.
- Fix: unparseable `RRULE` values were serialized with `TEXT` escaping, turning the structural `;` and `,` separators of the `RECUR` value type into `\;` and `\,` (RFC 5545, Section 3.3.10).
- Fix: date-only vCard 3.0 `REV` values were serialized with a fabricated `T000000` time, although a `date` value is valid there (RFC 2426, Section 3.6.4).
- Fix: truncated iCalendar `DATE-TIME` and vCard `TIMESTAMP` values were not completed, so a value missing its seconds did not survive a round trip, and a value missing its minutes was silently downgraded to a `DATE`, discarding the time.
- Fix: control characters were retained in parameter values and emitted verbatim by URI parameters such as `ALTREP` and `DIR`, although `QSAFE-CHAR` does not permit them (RFC 5545, Section 3.1).
- Fix: vCard content lines lacking a value separator were parsed with no value but serialized with an empty one.
- Fix: serialized lines exceeded the 75 octet limit by one because the colon separating a property from its value was not counted.
- Fix: an empty value at a fold boundary emitted a continuation line holding nothing but the fold character.
- Fix: vCard 2.1 `BASE64` values continued on unindented lines and terminated by a blank line were truncated to their first line, discarding the rest of the encoded data.
- Fix: an `RRULE` holding an invalid rule part silently yielded a partial recurrence rule, since every part after the invalid one was skipped and the resulting rule was returned as if it had parsed cleanly.
- Expanded the iCalendar and vCard round-trip test corpus with samples collected from public test suites.

calcard 0.3.10
================================
- Bump `mail-builder` dependency to 0.5.
- Use `rkyv::primitive::ArchivedU32` instead of the concrete `rkyv::rend::u32_le`.

calcard 0.3.9
================================
- Updated JSCalendar conversion rules according to `draft-ietf-calext-jscalendar-icalendar-25`.
- Fix: `ORGANIZER` converted to a duplicate Participant object instead of merging with the `ATTENDEE` or `PARTICIPANT` sharing its calendar address (#24).
- Fix: the `owner` role was not set on the Participant object converted from `ORGANIZER`.
- Fix: `iCalendar` converted properties of Link and Participant objects were matched position rather than by object key when no `JSID` parameter was present.
- Fix: all-day events exported as a floating DATE-TIME plus `SHOW-WITHOUT-TIME` instead of the `DATE` value type.
- Fix: `VERSION` was missing from the generated `VCALENDAR` component, and the JSCalendar `version` property was exported as a `JSPROP` property.

calcard 0.3.8
================================
- Fix: binary values without a media type serialized as `data:base64,`, which is not a valid
  `data` URL (RFC 2397).
- Map the legacy vCard 3.0 `TYPE` parameter of `PHOTO`, `LOGO`, `SOUND` and `KEY` to the media
  type of their binary value, so upgrading a v3.0 card to v4.0 yields `data:image/jpeg;base64,`
  instead of a typeless `data` URL.
- Include `TYPE=` when exporting binary values to vCard v3.0 and below.
- Fix: archived iCalendar writer escaped `,` and `;` inside quoted URI parameter values such as
  `DIR`.

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
