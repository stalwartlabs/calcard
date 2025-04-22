use crate::common::PartialDateTime;
use chrono::{DateTime, FixedOffset, Offset, TimeZone, Utc};
use hashify::tiny_map;
use std::str::FromStr;

use super::DateTimeResult;

#[derive(Clone, Copy)]
pub enum Tz {
    Floating,
    Fixed(FixedOffset),
    Tz(chrono_tz::Tz),
}

impl PartialDateTime {
    pub fn to_date_time_with_tz(&self, tz: Tz) -> Option<DateTime<Tz>> {
        self.to_date_time()
            .and_then(|dt| dt.to_date_time_with_tz(tz))
    }
}

impl DateTimeResult {
    pub fn to_date_time_with_tz(&self, tz: Tz) -> Option<DateTime<Tz>> {
        if let Some(offset) = self.offset {
            if offset.local_minus_utc() == 0 {
                Tz::UTC.from_utc_datetime(&self.date_time).into()
            } else {
                Tz::Fixed(offset)
                    .from_local_datetime(&self.date_time)
                    .single()
            }
        } else {
            tz.from_local_datetime(&self.date_time).single()
        }
    }
}

impl Tz {
    pub fn name(&self) -> &str {
        match self {
            Self::Floating => "Floating",
            Self::Tz(tz) => tz.name(),
            Self::Fixed(_) => "Fixed",
        }
    }

    pub fn is_floating(&self) -> bool {
        matches!(self, Self::Floating)
    }

    pub const UTC: Self = Self::Tz(chrono_tz::UTC);

    pub fn from_ms_cdo_zone_id(id: &str) -> Option<Self> {
        // Source https://learn.microsoft.com/en-us/previous-versions/office/developer/exchange-server-2007/aa563018(v=exchg.80)

        tiny_map!(id.as_bytes(),
            "0" => chrono_tz::Tz::UTC,
            "1" => chrono_tz::Tz::Europe__London,
            "10" => chrono_tz::Tz::America__New_York,
            "11" => chrono_tz::Tz::America__Chicago,
            "12" => chrono_tz::Tz::America__Denver,
            "13" => chrono_tz::Tz::America__Los_Angeles,
            "14" => chrono_tz::Tz::America__Anchorage,
            "15" => chrono_tz::Tz::Pacific__Honolulu,
            "16" => chrono_tz::Tz::Pacific__Midway,
            "17" => chrono_tz::Tz::Pacific__Auckland,
            "18" => chrono_tz::Tz::Australia__Brisbane,
            "19" => chrono_tz::Tz::Australia__Adelaide,
            "2" => chrono_tz::Tz::Europe__Lisbon,
            "20" => chrono_tz::Tz::Asia__Tokyo,
            "21" => chrono_tz::Tz::Asia__Singapore,
            "22" => chrono_tz::Tz::Asia__Bangkok,
            "23" => chrono_tz::Tz::Asia__Calcutta,
            "24" => chrono_tz::Tz::Asia__Muscat,
            "25" => chrono_tz::Tz::Asia__Tehran,
            "26" => chrono_tz::Tz::Asia__Baghdad,
            "27" => chrono_tz::Tz::Asia__Jerusalem,
            "28" => chrono_tz::Tz::America__St_Johns,
            "29" => chrono_tz::Tz::Atlantic__Azores,
            "3" => chrono_tz::Tz::Europe__Paris,
            "30" => chrono_tz::Tz::America__Noronha,
            "31" => chrono_tz::Tz::Africa__Casablanca,
            "32" => chrono_tz::Tz::America__Argentina__Buenos_Aires,
            "33" => chrono_tz::Tz::America__Caracas,
            "34" => chrono_tz::Tz::America__Indiana__Indianapolis,
            "35" => chrono_tz::Tz::America__Bogota,
            "36" => chrono_tz::Tz::America__Edmonton,
            "37" => chrono_tz::Tz::America__Mexico_City,
            "38" => chrono_tz::Tz::America__Phoenix,
            "39" => chrono_tz::Tz::Pacific__Kwajalein,
            "4" => chrono_tz::Tz::Europe__Berlin,
            "40" => chrono_tz::Tz::Pacific__Fiji,
            "41" => chrono_tz::Tz::Asia__Magadan,
            "42" => chrono_tz::Tz::Australia__Hobart,
            "43" => chrono_tz::Tz::Pacific__Guam,
            "44" => chrono_tz::Tz::Australia__Darwin,
            "45" => chrono_tz::Tz::Asia__Shanghai,
            "46" => chrono_tz::Tz::Asia__Almaty,
            "47" => chrono_tz::Tz::Asia__Karachi,
            "48" => chrono_tz::Tz::Asia__Kabul,
            "49" => chrono_tz::Tz::Africa__Cairo,
            "5" => chrono_tz::Tz::Europe__Bucharest,
            "50" => chrono_tz::Tz::Africa__Harare,
            "51" => chrono_tz::Tz::Europe__Moscow,
            "53" => chrono_tz::Tz::Atlantic__Cape_Verde,
            "54" => chrono_tz::Tz::Asia__Baku,
            "55" => chrono_tz::Tz::America__Guatemala,
            "56" => chrono_tz::Tz::Africa__Nairobi,
            "58" => chrono_tz::Tz::Asia__Yekaterinburg,
            "59" => chrono_tz::Tz::Europe__Helsinki,
            "6" => chrono_tz::Tz::Europe__Prague,
            "60" => chrono_tz::Tz::America__Godthab,
            "61" => chrono_tz::Tz::Asia__Rangoon,
            "62" => chrono_tz::Tz::Asia__Kathmandu,
            "63" => chrono_tz::Tz::Asia__Irkutsk,
            "64" => chrono_tz::Tz::Asia__Krasnoyarsk,
            "65" => chrono_tz::Tz::America__Santiago,
            "66" => chrono_tz::Tz::Asia__Colombo,
            "67" => chrono_tz::Tz::Pacific__Tongatapu,
            "68" => chrono_tz::Tz::Asia__Vladivostok,
            "69" => chrono_tz::Tz::Africa__Luanda,
            "7" => chrono_tz::Tz::Europe__Athens,
            "70" => chrono_tz::Tz::Asia__Yakutsk,
            "71" => chrono_tz::Tz::Asia__Dhaka,
            "72" => chrono_tz::Tz::Asia__Seoul,
            "73" => chrono_tz::Tz::Australia__Perth,
            "74" => chrono_tz::Tz::Asia__Kuwait,
            "75" => chrono_tz::Tz::Asia__Taipei,
            "76" => chrono_tz::Tz::Australia__Sydney,
            "8" => chrono_tz::Tz::America__Sao_Paulo,
            "9" => chrono_tz::Tz::America__Halifax,
        )
        .map(Self::Tz)
    }
}

impl FromStr for Tz {
    type Err = ();

    /*
      Calconnect recommends ignoring VTIMEZONE and just "guess" the timezone

      See: https://standards.calconnect.org/cc/cc-r0602-2006.html

    */

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // First try with the chrono_tz::Tz
        if let Ok(tz) = chrono_tz::Tz::from_str(s) {
            return Ok(Self::Tz(tz));
        }

        // Strip common prefixes
        let mut s = s.trim();
        let mut zone_offset = None;
        let mut retry_chrono_tz = false;
        if let Some(part) = s.strip_prefix('(') {
            if let Some((zone, name)) = part.split_once(')') {
                s = name.trim_start();
                zone_offset = Some(zone.trim());
            }
        } else if let Some(mut name) = s.strip_prefix('/') {
            /*
               The presence of the SOLIDUS character as a prefix, indicates that
               this "TZID" represents a unique ID in a globally defined time zone
               registry (when such registry is defined).
            */
            if name.as_bytes().iter().filter(|&&c| c == b'/').count() > 2 {
                // Extract zones such as '/softwarestudio.org/Olson_20011030_5/America/Chicago'
                if let Some(new_name) = name.splitn(3, '/').nth(2) {
                    name = new_name.strip_prefix("SystemV/").unwrap_or(new_name);
                }
            }
            retry_chrono_tz = true;
            s = name;
        }

        // Try again with chrono_tz::Tz
        if retry_chrono_tz {
            if let Ok(tz) = chrono_tz::Tz::from_str(s) {
                return Ok(Self::Tz(tz));
            }
        }

        // Map propietary timezones to chrono_tz::Tz
        let result = hashify::map!(s.as_bytes(), chrono_tz::Tz,
        "AUS Central Standard Time" => chrono_tz::Tz::Australia__Darwin,
        "AUS Central" => chrono_tz::Tz::Australia__Darwin,
        "AUS Eastern Standard Time" => chrono_tz::Tz::Australia__Sydney,
        "AUS Eastern" => chrono_tz::Tz::Australia__Sydney,
        "Abu Dhabi, Muscat" => chrono_tz::Tz::Asia__Muscat,
        "Adelaide, Central Australia" => chrono_tz::Tz::Australia__Adelaide,
        "Afghanistan Standard Time" => chrono_tz::Tz::Asia__Kabul,
        "Afghanistan" => chrono_tz::Tz::Asia__Kabul,
        "Alaska" => chrono_tz::Tz::America__Anchorage,
        "Alaskan Standard Time" => chrono_tz::Tz::America__Anchorage,
        "Alaskan" => chrono_tz::Tz::America__Anchorage,
        "Aleutian Standard Time" => chrono_tz::Tz::America__Adak,
        "Almaty, Novosibirsk, North Central Asia" => chrono_tz::Tz::Asia__Almaty,
        "Altai Standard Time" => chrono_tz::Tz::Asia__Barnaul,
        "Amsterdam, Berlin, Bern, Rom, Stockholm, Wien" => chrono_tz::Tz::Europe__Berlin,
        "Amsterdam, Berlin, Bern, Rome, Stockholm, Vienna" => chrono_tz::Tz::Europe__Berlin,
        "Arab Standard Time" => chrono_tz::Tz::Asia__Riyadh,
        "Arab" => chrono_tz::Tz::Asia__Kuwait,
        "Arab, Kuwait, Riyadh" => chrono_tz::Tz::Asia__Kuwait,
        "Arabian Standard Time" => chrono_tz::Tz::Asia__Dubai,
        "Arabian" => chrono_tz::Tz::Asia__Muscat,
        "Arabic Standard Time" => chrono_tz::Tz::Asia__Baghdad,
        "Arabic" => chrono_tz::Tz::Asia__Baghdad,
        "Argentina Standard Time" => chrono_tz::Tz::America__Buenos_Aires,
        "Argentina" => chrono_tz::Tz::America__Argentina__Buenos_Aires,
        "Arizona" => chrono_tz::Tz::America__Phoenix,
        "Armenian" => chrono_tz::Tz::Asia__Yerevan,
        "Astana, Dhaka" => chrono_tz::Tz::Asia__Dhaka,
        "Astrakhan Standard Time" => chrono_tz::Tz::Europe__Astrakhan,
        "Athens, Istanbul, Minsk" => chrono_tz::Tz::Europe__Athens,
        "Atlantic Standard Time" => chrono_tz::Tz::America__Halifax,
        "Atlantic Time (Canada)" => chrono_tz::Tz::America__Halifax,
        "Atlantic" => chrono_tz::Tz::America__Halifax,
        "Auckland, Wellington" => chrono_tz::Tz::Pacific__Auckland,
        "Aus Central W. Standard Time" => chrono_tz::Tz::Australia__Eucla,
        "Azerbaijan Standard Time" => chrono_tz::Tz::Asia__Baku,
        "Azerbijan" => chrono_tz::Tz::Asia__Baku,
        "Azores Standard Time" => chrono_tz::Tz::Atlantic__Azores,
        "Azores" => chrono_tz::Tz::Atlantic__Azores,
        "Baghdad" => chrono_tz::Tz::Asia__Baghdad,
        "Bahia Standard Time" => chrono_tz::Tz::America__Bahia,
        "Baku, Tbilisi, Yerevan" => chrono_tz::Tz::Asia__Baku,
        "Bangkok, Hanoi, Jakarta" => chrono_tz::Tz::Asia__Bangkok,
        "Bangladesh Standard Time" => chrono_tz::Tz::Asia__Dhaka,
        "Beijing, Chongqing, Hong Kong SAR, Urumqi" => chrono_tz::Tz::Asia__Shanghai,
        "Belarus Standard Time" => chrono_tz::Tz::Europe__Minsk,
        "Belgrade, Pozsony, Budapest, Ljubljana, Prague" => chrono_tz::Tz::Europe__Prague,
        "Bogota, Lima, Quito" => chrono_tz::Tz::America__Bogota,
        "Bougainville Standard Time" => chrono_tz::Tz::Pacific__Bougainville,
        "Brasilia" => chrono_tz::Tz::America__Sao_Paulo,
        "Brisbane, East Australia" => chrono_tz::Tz::Australia__Brisbane,
        "Brussels, Copenhagen, Madrid, Paris" => chrono_tz::Tz::Europe__Paris,
        "Bucharest" => chrono_tz::Tz::Europe__Bucharest,
        "Buenos Aires" => chrono_tz::Tz::America__Argentina__Buenos_Aires,
        "Cairo" => chrono_tz::Tz::Africa__Cairo,
        "Canada Central Standard Time" => chrono_tz::Tz::America__Regina,
        "Canada Central" => chrono_tz::Tz::America__Edmonton,
        "Canberra, Melbourne, Sydney, Hobart" => chrono_tz::Tz::Australia__Sydney,
        "Canberra, Melbourne, Sydney" => chrono_tz::Tz::Australia__Sydney,
        "Cape Verde Is." => chrono_tz::Tz::Atlantic__Cape_Verde,
        "Cape Verde Standard Time" => chrono_tz::Tz::Atlantic__Cape_Verde,
        "Cape Verde" => chrono_tz::Tz::Atlantic__Cape_Verde,
        "Caracas, La Paz" => chrono_tz::Tz::America__Caracas,
        "Casablanca, Monrovia" => chrono_tz::Tz::Africa__Casablanca,
        "Caucasus Standard Time" => chrono_tz::Tz::Asia__Yerevan,
        "Caucasus" => chrono_tz::Tz::Asia__Yerevan,
        "Cen. Australia Standard Time" => chrono_tz::Tz::Australia__Adelaide,
        "Cen. Australia" => chrono_tz::Tz::Australia__Adelaide,
        "Central America Standard Time" => chrono_tz::Tz::America__Guatemala,
        "Central America" => chrono_tz::Tz::America__Guatemala,
        "Central Asia Standard Time" => chrono_tz::Tz::Asia__Almaty,
        "Central Asia" => chrono_tz::Tz::Asia__Dhaka,
        "Central Brazilian Standard Time" => chrono_tz::Tz::America__Cuiaba,
        "Central Brazilian" => chrono_tz::Tz::America__Manaus,
        "Central Europe Standard Time" => chrono_tz::Tz::Europe__Budapest,
        "Central Europe" => chrono_tz::Tz::Europe__Prague,
        "Central European Standard Time" => chrono_tz::Tz::Europe__Warsaw,
        "Central European" => chrono_tz::Tz::Europe__Sarajevo,
        "Central Pacific Standard Time" => chrono_tz::Tz::Pacific__Guadalcanal,
        "Central Pacific" => chrono_tz::Tz::Asia__Magadan,
        "Central Standard Time (Mexico)" => chrono_tz::Tz::America__Mexico_City,
        "Central Standard Time" => chrono_tz::Tz::America__Chicago,
        "Central Time (US & Canada)" => chrono_tz::Tz::America__Chicago,
        "Central" => chrono_tz::Tz::America__Chicago,
        "Chatham Islands Standard Time" => chrono_tz::Tz::Pacific__Chatham,
        "China Standard Time" => chrono_tz::Tz::Asia__Shanghai,
        "China" => chrono_tz::Tz::Asia__Shanghai,
        "Cuba Standard Time" => chrono_tz::Tz::America__Havana,
        "Darwin" => chrono_tz::Tz::Australia__Darwin,
        "Dateline Standard Time" => chrono_tz::Tz::Etc__GMTPlus12,
        "Dateline" => chrono_tz::Tz::Etc__GMTMinus12,
        "E. Africa Standard Time" => chrono_tz::Tz::Africa__Nairobi,
        "E. Africa" => chrono_tz::Tz::Africa__Nairobi,
        "E. Australia Standard Time" => chrono_tz::Tz::Australia__Brisbane,
        "E. Australia" => chrono_tz::Tz::Australia__Brisbane,
        "E. Europe Standard Time" => chrono_tz::Tz::Europe__Chisinau,
        "E. Europe" => chrono_tz::Tz::Europe__Minsk,
        "E. South America Standard Time" => chrono_tz::Tz::America__Sao_Paulo,
        "E. South America" => chrono_tz::Tz::America__Belem,
        "East Africa, Nairobi" => chrono_tz::Tz::Africa__Nairobi,
        "Easter Island Standard Time" => chrono_tz::Tz::Pacific__Easter,
        "Eastern Standard Time (Mexico)" => chrono_tz::Tz::America__Cancun,
        "Eastern Standard Time" => chrono_tz::Tz::America__New_York,
        "Eastern Time (US & Canada)" => chrono_tz::Tz::America__New_York,
        "Eastern" => chrono_tz::Tz::America__New_York,
        "Egypt Standard Time" => chrono_tz::Tz::Africa__Cairo,
        "Egypt" => chrono_tz::Tz::Africa__Cairo,
        "Ekaterinburg Standard Time" => chrono_tz::Tz::Asia__Yekaterinburg,
        "Ekaterinburg" => chrono_tz::Tz::Asia__Yekaterinburg,
        "Eniwetok, Kwajalein, Dateline Time" => chrono_tz::Tz::Pacific__Kwajalein,
        "FLE Standard Time" => chrono_tz::Tz::Europe__Kiev,
        "FLE" => chrono_tz::Tz::Europe__Helsinki,
        "Fiji Islands, Kamchatka, Marshall Is." => chrono_tz::Tz::Pacific__Fiji,
        "Fiji Standard Time" => chrono_tz::Tz::Pacific__Fiji,
        "Fiji" => chrono_tz::Tz::Pacific__Fiji,
        "GMT Standard Time" => chrono_tz::Tz::Europe__London,
        "GTB Standard Time" => chrono_tz::Tz::Europe__Bucharest,
        "GTB" => chrono_tz::Tz::Europe__Athens,
        "Georgian Standard Time" => chrono_tz::Tz::Asia__Tbilisi,
        "Georgian" => chrono_tz::Tz::Asia__Tbilisi,
        "Greenland Standard Time" => chrono_tz::Tz::America__Godthab,
        "Greenland" => chrono_tz::Tz::America__Godthab,
        "Greenwich Mean Time: Dublin, Edinburgh, Lisbon, London" => chrono_tz::Tz::Europe__Lisbon,
        "Greenwich Mean Time; Dublin, Edinburgh, London" => chrono_tz::Tz::Europe__London,
        "Greenwich Standard Time" => chrono_tz::Tz::Atlantic__Reykjavik,
        "Greenwich" => chrono_tz::Tz::Atlantic__Reykjavik,
        "Guam, Port Moresby" => chrono_tz::Tz::Pacific__Guam,
        "Haiti Standard Time" => chrono_tz::Tz::America__PortauPrince,
        "Harare, Pretoria" => chrono_tz::Tz::Africa__Harare,
        "Hawaii" => chrono_tz::Tz::Pacific__Honolulu,
        "Hawaiian Standard Time" => chrono_tz::Tz::Pacific__Honolulu,
        "Hawaiian" => chrono_tz::Tz::Pacific__Honolulu,
        "Helsinki, Riga, Tallinn" => chrono_tz::Tz::Europe__Helsinki,
        "Hobart, Tasmania" => chrono_tz::Tz::Australia__Hobart,
        "India Standard Time" => chrono_tz::Tz::Asia__Calcutta,
        "India" => chrono_tz::Tz::Asia__Calcutta,
        "Indiana (East)" => chrono_tz::Tz::America__Indiana__Indianapolis,
        "Iran Standard Time" => chrono_tz::Tz::Asia__Tehran,
        "Iran" => chrono_tz::Tz::Asia__Tehran,
        "Irkutsk, Ulaan Bataar" => chrono_tz::Tz::Asia__Irkutsk,
        "Islamabad, Karachi, Tashkent" => chrono_tz::Tz::Asia__Karachi,
        "Israel Standard Time" => chrono_tz::Tz::Asia__Jerusalem,
        "Israel" => chrono_tz::Tz::Asia__Jerusalem,
        "Israel, Jerusalem Standard Time" => chrono_tz::Tz::Asia__Jerusalem,
        "Jordan Standard Time" => chrono_tz::Tz::Asia__Amman,
        "Jordan" => chrono_tz::Tz::Asia__Amman,
        "Kabul" => chrono_tz::Tz::Asia__Kabul,
        "Kaliningrad Standard Time" => chrono_tz::Tz::Europe__Kaliningrad,
        "Kathmandu, Nepal" => chrono_tz::Tz::Asia__Kathmandu,
        "Kolkata, Chennai, Mumbai, New Delhi, India Standard Time" => chrono_tz::Tz::Asia__Calcutta,
        "Korea Standard Time" => chrono_tz::Tz::Asia__Seoul,
        "Korea" => chrono_tz::Tz::Asia__Seoul,
        "Krasnoyarsk" => chrono_tz::Tz::Asia__Krasnoyarsk,
        "Kuala Lumpur, Singapore" => chrono_tz::Tz::Asia__Singapore,
        "Libya Standard Time" => chrono_tz::Tz::Africa__Tripoli,
        "Line Islands Standard Time" => chrono_tz::Tz::Pacific__Kiritimati,
        "Lord Howe Standard Time" => chrono_tz::Tz::Australia__Lord_Howe,
        "Magadan Standard Time" => chrono_tz::Tz::Asia__Magadan,
        "Magadan, Solomon Is., New Caledonia" => chrono_tz::Tz::Asia__Magadan,
        "Magallanes Standard Time" => chrono_tz::Tz::America__Punta_Arenas,
        "Marquesas Standard Time" => chrono_tz::Tz::Pacific__Marquesas,
        "Mauritius Standard Time" => chrono_tz::Tz::Indian__Mauritius,
        "Mauritius" => chrono_tz::Tz::Indian__Mauritius,
        "Mexico City, Tegucigalpa" => chrono_tz::Tz::America__Mexico_City,
        "Mexico Standard Time 2" => chrono_tz::Tz::America__Chihuahua,
        "Mexico" => chrono_tz::Tz::America__Mexico_City,
        "Mid-Atlantic" => chrono_tz::Tz::America__Noronha,
        "Middle East Standard Time" => chrono_tz::Tz::Asia__Beirut,
        "Middle East" => chrono_tz::Tz::Asia__Beirut,
        "Midway Island, Samoa" => chrono_tz::Tz::Pacific__Midway,
        "Montevideo Standard Time" => chrono_tz::Tz::America__Montevideo,
        "Montevideo" => chrono_tz::Tz::America__Montevideo,
        "Morocco Standard Time" => chrono_tz::Tz::Africa__Casablanca,
        "Morocco" => chrono_tz::Tz::Africa__Casablanca,
        "Moscow, St. Petersburg, Volgograd" => chrono_tz::Tz::Europe__Moscow,
        "Mountain Standard Time (Mexico)" => chrono_tz::Tz::America__Chihuahua,
        "Mountain Standard Time" => chrono_tz::Tz::America__Denver,
        "Mountain Time (US & Canada)" => chrono_tz::Tz::America__Denver,
        "Mountain" => chrono_tz::Tz::America__Denver,
        "Myanmar Standard Time" => chrono_tz::Tz::Asia__Rangoon,
        "Myanmar" => chrono_tz::Tz::Asia__Rangoon,
        "N. Central Asia Standard Time" => chrono_tz::Tz::Asia__Novosibirsk,
        "N. Central Asia" => chrono_tz::Tz::Asia__Almaty,
        "Namibia Standard Time" => chrono_tz::Tz::Africa__Windhoek,
        "Namibia" => chrono_tz::Tz::Africa__Windhoek,
        "Nepal Standard Time" => chrono_tz::Tz::Asia__Katmandu,
        "Nepal" => chrono_tz::Tz::Asia__Kathmandu,
        "New Zealand Standard Time" => chrono_tz::Tz::Pacific__Auckland,
        "New Zealand" => chrono_tz::Tz::Pacific__Auckland,
        "Newfoundland Standard Time" => chrono_tz::Tz::America__St_Johns,
        "Newfoundland" => chrono_tz::Tz::America__St_Johns,
        "Norfolk Standard Time" => chrono_tz::Tz::Pacific__Norfolk,
        "North Asia East Standard Time" => chrono_tz::Tz::Asia__Irkutsk,
        "North Asia East" => chrono_tz::Tz::Asia__Irkutsk,
        "North Asia Standard Time" => chrono_tz::Tz::Asia__Krasnoyarsk,
        "North Asia" => chrono_tz::Tz::Asia__Krasnoyarsk,
        "North Korea Standard Time" => chrono_tz::Tz::Asia__Pyongyang,
        "Omsk Standard Time" => chrono_tz::Tz::Asia__Omsk,
        "Osaka, Sapporo, Tokyo" => chrono_tz::Tz::Asia__Tokyo,
        "Japan" => chrono_tz::Tz::Asia__Tokyo,
        "Pacific SA Standard Time" => chrono_tz::Tz::America__Santiago,
        "Pacific SA" => chrono_tz::Tz::America__Santiago,
        "Pacific Standard Time (Mexico)" => chrono_tz::Tz::America__Tijuana,
        "Pacific Standard Time" => chrono_tz::Tz::America__Los_Angeles,
        "Pacific Time (US & Canada)" => chrono_tz::Tz::America__Los_Angeles,
        "Pacific Time (US & Canada); Tijuana" => chrono_tz::Tz::America__Los_Angeles,
        "Pacific" => chrono_tz::Tz::America__Los_Angeles,
        "Pakistan Standard Time" => chrono_tz::Tz::Asia__Karachi,
        "Pakistan" => chrono_tz::Tz::Asia__Karachi,
        "Paraguay Standard Time" => chrono_tz::Tz::America__Asuncion,
        "Paris, Madrid, Brussels, Copenhagen" => chrono_tz::Tz::Europe__Paris,
        "Perth, Western Australia" => chrono_tz::Tz::Australia__Perth,
        "Prague, Central Europe" => chrono_tz::Tz::Europe__Prague,
        "Qyzylorda Standard Time" => chrono_tz::Tz::Asia__Qyzylorda,
        "Rangoon" => chrono_tz::Tz::Asia__Rangoon,
        "Romance Standard Time" => chrono_tz::Tz::Europe__Paris,
        "Romance" => chrono_tz::Tz::Europe__Paris,
        "Russia Time Zone 10" => chrono_tz::Tz::Asia__Srednekolymsk,
        "Russia Time Zone 11" => chrono_tz::Tz::Asia__Kamchatka,
        "Russia Time Zone 3" => chrono_tz::Tz::Europe__Samara,
        "Russian Standard Time" => chrono_tz::Tz::Europe__Moscow,
        "Russian" => chrono_tz::Tz::Europe__Moscow,
        "SA Eastern Standard Time" => chrono_tz::Tz::America__Cayenne,
        "SA Eastern" => chrono_tz::Tz::America__Belem,
        "SA Pacific Standard Time" => chrono_tz::Tz::America__Bogota,
        "SA Pacific" => chrono_tz::Tz::America__Bogota,
        "SA Western Standard Time" => chrono_tz::Tz::America__La_Paz,
        "SA Western" => chrono_tz::Tz::America__La_Paz,
        "SE Asia Standard Time" => chrono_tz::Tz::Asia__Bangkok,
        "SE Asia" => chrono_tz::Tz::Asia__Bangkok,
        "Saint Pierre Standard Time" => chrono_tz::Tz::America__Miquelon,
        "Sakhalin Standard Time" => chrono_tz::Tz::Asia__Sakhalin,
        "Samoa Standard Time" => chrono_tz::Tz::Pacific__Apia,
        "Samoa" => chrono_tz::Tz::Pacific__Apia,
        "Santiago" => chrono_tz::Tz::America__Santiago,
        "Sao Tome Standard Time" => chrono_tz::Tz::Africa__Sao_Tome,
        "Sarajevo, Skopje, Sofija, Vilnius, Warsaw, Zagreb" => chrono_tz::Tz::Europe__Sarajevo,
        "Saratov Standard Time" => chrono_tz::Tz::Europe__Saratov,
        "Saskatchewan" => chrono_tz::Tz::America__Edmonton,
        "Seoul, Korea Standard time" => chrono_tz::Tz::Asia__Seoul,
        "Singapore Standard Time" => chrono_tz::Tz::Asia__Singapore,
        "Singapore" => chrono_tz::Tz::Asia__Singapore,
        "South Africa Standard Time" => chrono_tz::Tz::Africa__Johannesburg,
        "South Africa" => chrono_tz::Tz::Africa__Harare,
        "Sri Jayawardenepura, Sri Lanka" => chrono_tz::Tz::Asia__Colombo,
        "Sri Lanka Standard Time" => chrono_tz::Tz::Asia__Colombo,
        "Sri Lanka" => chrono_tz::Tz::Asia__Colombo,
        "Sudan Standard Time" => chrono_tz::Tz::Africa__Khartoum,
        "Syria Standard Time" => chrono_tz::Tz::Asia__Damascus,
        "Taipei Standard Time" => chrono_tz::Tz::Asia__Taipei,
        "Taipei" => chrono_tz::Tz::Asia__Taipei,
        "Tasmania Standard Time" => chrono_tz::Tz::Australia__Hobart,
        "Tasmania" => chrono_tz::Tz::Australia__Hobart,
        "Tehran" => chrono_tz::Tz::Asia__Tehran,
        "Tocantins Standard Time" => chrono_tz::Tz::America__Araguaina,
        "Tokyo Standard Time" => chrono_tz::Tz::Asia__Tokyo,
        "Tokyo" => chrono_tz::Tz::Asia__Tokyo,
        "Tomsk Standard Time" => chrono_tz::Tz::Asia__Tomsk,
        "Tonga Standard Time" => chrono_tz::Tz::Pacific__Tongatapu,
        "Tonga" => chrono_tz::Tz::Pacific__Tongatapu,
        "Transbaikal Standard Time" => chrono_tz::Tz::Asia__Chita,
        "Turkey Standard Time" => chrono_tz::Tz::Europe__Istanbul,
        "Turks And Caicos Standard Time" => chrono_tz::Tz::America__Grand_Turk,
        "US Eastern Standard Time" => chrono_tz::Tz::America__Indianapolis,
        "US Eastern" => chrono_tz::Tz::America__Indiana__Indianapolis,
        "US Mountain Standard Time" => chrono_tz::Tz::America__Phoenix,
        "US Mountain" => chrono_tz::Tz::America__Phoenix,
        "UTC" => chrono_tz::Tz::Etc__GMT,
        "UTC+12" => chrono_tz::Tz::Etc__GMTMinus12,
        "UTC+13" => chrono_tz::Tz::Etc__GMTMinus13,
        "UTC-02" => chrono_tz::Tz::Etc__GMTPlus2,
        "UTC-08" => chrono_tz::Tz::Etc__GMTPlus8,
        "UTC-09" => chrono_tz::Tz::Etc__GMTPlus9,
        "UTC-11" => chrono_tz::Tz::Etc__GMTPlus11,
        "Ulaanbaatar Standard Time" => chrono_tz::Tz::Asia__Ulaanbaatar,
        "Universal Coordinated Time" => chrono_tz::Tz::UTC,
        "Venezuela Standard Time" => chrono_tz::Tz::America__Caracas,
        "Venezuela" => chrono_tz::Tz::America__Caracas,
        "Vladivostok Standard Time" => chrono_tz::Tz::Asia__Vladivostok,
        "Vladivostok" => chrono_tz::Tz::Asia__Vladivostok,
        "Volgograd Standard Time" => chrono_tz::Tz::Europe__Volgograd,
        "W. Australia Standard Time" => chrono_tz::Tz::Australia__Perth,
        "W. Australia" => chrono_tz::Tz::Australia__Perth,
        "W. Central Africa Standard Time" => chrono_tz::Tz::Africa__Lagos,
        "W. Central Africa" => chrono_tz::Tz::Africa__Lagos,
        "W. Europe Standard Time" => chrono_tz::Tz::Europe__Berlin,
        "W. Europe" => chrono_tz::Tz::Europe__Amsterdam,
        "W. Mongolia Standard Time" => chrono_tz::Tz::Asia__Hovd,
        "West Asia Standard Time" => chrono_tz::Tz::Asia__Tashkent,
        "West Asia" => chrono_tz::Tz::Asia__Tashkent,
        "West Bank Standard Time" => chrono_tz::Tz::Asia__Hebron,
        "West Central Africa" => chrono_tz::Tz::Africa__Luanda,
        "West Pacific Standard Time" => chrono_tz::Tz::Pacific__Port_Moresby,
        "West Pacific" => chrono_tz::Tz::Pacific__Guam,
        "Yakutsk Standard Time" => chrono_tz::Tz::Asia__Yakutsk,
        "Yakutsk" => chrono_tz::Tz::Asia__Yakutsk,
        "Yukon Standard Time" => chrono_tz::Tz::America__Whitehorse,
        "Nuku'alofa, Tonga" => chrono_tz::Tz::Pacific__Tongatapu,
        );

        if let Some(tz) = result {
            return Ok(Self::Tz(*tz));
        } else if let Some(zone_offset) = zone_offset {
            let (zone, sign, time) = if let Some((zone, part)) = zone_offset.split_once('+') {
                (zone.trim(), '+', part.trim())
            } else if let Some((zone, part)) = zone_offset.split_once('-') {
                (zone.trim(), '-', part.trim())
            } else {
                return Err(());
            };
            if !zone.eq_ignore_ascii_case("UTC") && !zone.eq_ignore_ascii_case("GMT") {
                return Err(());
            }
            let mut zone = String::with_capacity(10);
            zone.push_str("Etc/GMT");
            zone.push(sign);

            for (pos, ch) in time.chars().enumerate() {
                if !ch.is_ascii_digit() {
                    break;
                } else if ch != '0' || pos > 0 {
                    zone.push(ch);
                }
            }

            if let Ok(tz) = chrono_tz::Tz::from_str(&zone) {
                return Ok(Self::Tz(tz));
            }
        }
        Err(())
    }
}

impl PartialEq for Tz {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Floating, Self::Floating) => true,
            (Self::Tz(l0), Self::Tz(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl From<Utc> for Tz {
    fn from(_tz: Utc) -> Self {
        Self::Tz(chrono_tz::UTC)
    }
}

impl From<chrono_tz::Tz> for Tz {
    fn from(tz: chrono_tz::Tz) -> Self {
        Self::Tz(tz)
    }
}

impl std::fmt::Debug for Tz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Floating => write!(f, "Floating"),
            Self::Tz(tz) => tz.fmt(f),
            Self::Fixed(fixed_offset) => fixed_offset.fmt(f),
        }
    }
}

impl std::fmt::Display for Tz {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Floating => write!(f, "Floating"),
            Self::Tz(tz) => tz.fmt(f),
            Self::Fixed(fixed_offset) => fixed_offset.fmt(f),
        }
    }
}

#[derive(Clone, Copy)]
pub enum RRuleOffset {
    Fixed(FixedOffset),
    Tz(<chrono_tz::Tz as TimeZone>::Offset),
    Floating,
}

impl std::fmt::Debug for RRuleOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed(offset) => offset.fmt(f),
            Self::Tz(offset) => offset.fmt(f),
            Self::Floating => write!(f, "Floating"),
        }
    }
}

impl std::fmt::Display for RRuleOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fixed(offset) => offset.fmt(f),
            Self::Tz(offset) => offset.fmt(f),
            Self::Floating => write!(f, "Floating"),
        }
    }
}

impl Offset for RRuleOffset {
    fn fix(&self) -> FixedOffset {
        match self {
            Self::Fixed(tz) => tz.fix(),
            Self::Tz(tz) => tz.fix(),
            Self::Floating => FixedOffset::east_opt(0).unwrap(),
        }
    }
}

impl TimeZone for Tz {
    type Offset = RRuleOffset;

    fn from_offset(offset: &Self::Offset) -> Self {
        match offset {
            RRuleOffset::Tz(offset) => Self::Tz(chrono_tz::Tz::from_offset(offset)),
            RRuleOffset::Fixed(fixed_offset) => Self::Fixed(*fixed_offset),
            RRuleOffset::Floating => Self::Floating,
        }
    }

    #[allow(deprecated)]
    fn offset_from_local_date(
        &self,
        local: &chrono::NaiveDate,
    ) -> chrono::LocalResult<Self::Offset> {
        match self {
            Self::Fixed(tz) => tz
                .from_local_date(local)
                .map(|date| RRuleOffset::Fixed(*date.offset())),
            Self::Tz(tz) => tz
                .from_local_date(local)
                .map(|date| RRuleOffset::Tz(*date.offset())),
            Self::Floating => chrono::LocalResult::Single(RRuleOffset::Floating),
        }
    }

    fn offset_from_local_datetime(
        &self,
        local: &chrono::NaiveDateTime,
    ) -> chrono::LocalResult<Self::Offset> {
        match self {
            Self::Fixed(tz) => tz
                .from_local_datetime(local)
                .map(|date| RRuleOffset::Fixed(*date.offset())),
            Self::Tz(tz) => tz
                .from_local_datetime(local)
                .map(|date| RRuleOffset::Tz(*date.offset())),
            Self::Floating => chrono::LocalResult::Single(RRuleOffset::Floating),
        }
    }

    #[allow(deprecated)]
    fn offset_from_utc_date(&self, utc: &chrono::NaiveDate) -> Self::Offset {
        match self {
            Self::Fixed(tz) => RRuleOffset::Fixed(*tz.from_utc_date(utc).offset()),
            Self::Tz(tz) => RRuleOffset::Tz(*tz.from_utc_date(utc).offset()),
            Self::Floating => RRuleOffset::Floating,
        }
    }

    fn offset_from_utc_datetime(&self, utc: &chrono::NaiveDateTime) -> Self::Offset {
        match self {
            Self::Fixed(tz) => RRuleOffset::Fixed(*tz.from_utc_datetime(utc).offset()),
            Self::Tz(tz) => RRuleOffset::Tz(*tz.from_utc_datetime(utc).offset()),
            Self::Floating => RRuleOffset::Floating,
        }
    }
}
