# calcard

[![crates.io](https://img.shields.io/crates/v/calcard)](https://crates.io/crates/calcard)
[![build](https://github.com/stalwartlabs/calcard/actions/workflows/rust.yml/badge.svg)](https://github.com/stalwartlabs/calcard/actions/workflows/rust.yml)
[![docs.rs](https://img.shields.io/docsrs/calcard)](https://docs.rs/calcard)
[![crates.io](https://img.shields.io/crates/l/calcard)](http://www.apache.org/licenses/LICENSE-2.0)

Calcard is a Rust crate for parsing and generating vCard and iCalendar data. It is designed to easily handle `.vcf` and `.ics` file formats, making it straightforward to integrate with calendaring and contact management applications.

In general, this library abides by the Postel's law or [Robustness Principle](https://en.wikipedia.org/wiki/Robustness_principle) which 
states that an implementation must be conservative in its sending behavior and liberal in its receiving behavior. This means that
_calcard_ will make a best effort to parse non-conformant iCal/vCard objects as long as these do not deviate too much from the standard.

## Usage

### Parsing Stream

You can parse vCard and iCalendar files using the stream-based parser:

```rust
let input = "BEGIN:VCARD\nVERSION:3.0\nFN:John Doe\nEND:VCARD\n";
let mut parser = Parser::new(&input);

loop {
    match parser.entry() {
        Entry::VCard(vcard) => println!("Parsed VCard: {:?}", vcard),
        Entry::ICalendar(ical) => println!("Parsed ICalendar: {:?}", ical),
        Entry::InvalidLine(line) => eprintln!("Invalid line found: {}", line),
        Entry::UnexpectedComponentEnd { expected, found } => {
            eprintln!("Unexpected end: expected {:?}, found {:?}", expected, found);
        },
        Entry::UnterminatedComponent(component) => {
            eprintln!("Unterminated component: {:?}", component);
        },
        Entry::Eof => {
            break;
        }
    }
}
```

### Parsing a Single vCard or iCalendar

You can also parse a single vCard or iCalendar instance directly:

```rust
let vcard_input = "BEGIN:VCARD\nVERSION:3.0\nFN:John Doe\nEND:VCARD\n";
let vcard = VCard::parse(&vcard_input);
println!("Parsed VCard: {:?}", vcard);

let ical_input = "BEGIN:VCALENDAR\nVERSION:2.0\nEND:VCALENDAR\n";
let ical = ICalendar::parse(&ical_input);
println!("Parsed ICalendar: {:?}", ical);
```

## Generation

To generate a vCard or iCalendar, simply call `.to_string()` on the parsed object:

```rust
let vcard_string = vcard.to_string();
println!("Generated vCard:\n{}", vcard_string);

let ical_string = ical.to_string();
println!("Generated ICalendar:\n{}", ical_string);
```

*Note: Documentation for creating VCard and ICalendar objects is coming soon.*

## Testing and Fuzzing

To run the testsuite:

```bash
 $ cargo test --all-features
```

or, to run the testsuite with MIRI:

```bash
 $ cargo +nightly miri test --all-features
```

To fuzz the library with `cargo-fuzz`:

```bash
 $ cargo +nightly fuzz run fuzz_all
 $ cargo +nightly fuzz run fuzz_random
 $ cargo +nightly fuzz run fuzz_structured
```

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Funding

Part of the development of this library was funded through the [NGI Zero Core](https://nlnet.nl/NGI0/), a fund established by [NLnet](https://nlnet.nl/) with financial support from the European Commission's programme, under the aegis of DG Communications Networks, Content and Technology under grant agreement No 101092990.

If you find this library useful you can help by [becoming a sponsor](https://opencollective.com/stalwart). Thank you!

## Copyright

Copyright (C) 2020, Stalwart Labs LLC
