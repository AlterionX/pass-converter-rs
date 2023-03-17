use std::{io::{Read, Seek}, collections::HashMap};

use zip::result::ZipResult;

pub enum PassFormat {
    GPass,
    PkPass,
}

#[derive(Debug, Clone)]
pub struct Pass {
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SubtypeKey {
    Flight,
}

impl SubtypeKey {
    const KEYS: [Self; 1] = [
        SubtypeKey::Flight,
    ];
}

trait PassSubtypeOwner {
    fn check_for_key(key: SubtypeKey, json: &serde_json::Map<String, serde_json::Value>) -> Option<&'static str>;

    /// Extracts a single subtype.
    fn extract_subtype<'a>(json: &'a serde_json::Value) -> Result<(SubtypeKey, &'a serde_json::Value), &'static str> {
        let mut found_information = vec![];

        let Some(obj) = json.as_object() else {
            return Err("JSON is not an object.");
        };

        for key in SubtypeKey::KEYS {
            let Some(subtype_key) = Self::check_for_key(key, obj) else {
                continue;
            };
            let Some(subtype_entry) = obj.get(subtype_key) else {
                continue;
            };

            found_information.push((key, subtype_entry));
        }

        match found_information.len() {
            0 => Err("Missing subtype"),
            1 => Ok(found_information.remove(0)),
            _ => Err("More than one subtype"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GoogleWalletPass {
}

#[derive(Debug, Clone)]
pub struct PkPass {
    // serde_json: serde_json::Value,
    // l10n: HashMap<String, HashMap<String, Vec<u8>>>,

    pub base: PkPassBase,
    pub barcode: PkPassBarcode,
    pub subtype: PkPassSubtype,
}

#[derive(Debug, Clone)]
pub struct PkPassBarcode {
    pub format: String,
    pub message: String,
    pub encoding: String,
}

impl PkPassBarcode {
    fn extract(json: &serde_json::Value, year: i32) -> Result<Self, anyhow::Error> {
        let Some(base_obj) = json.as_object() else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Expected json to be object.".to_owned()).into());
        };
        let Some(base_obj) = base_obj.get("barcode") else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBarcode is missing key".to_owned()).into());
        };
        let Some(obj) = base_obj.as_object() else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Expected barcode to be object.".to_owned()).into());
        };

        let Some(format) = obj.get("format").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBarcode is missing key".to_owned()).into());
        };
        let Some(message) = obj.get("message").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBarcode is missing key".to_owned()).into());
        };
        let Some(encoding) = obj.get("messageEncoding").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBarcode is missing key".to_owned()).into());
        };

        Ok(Self {
            format,
            message,
            encoding,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PkPassBase {
    // id: json.serial_number,
    // type_id: json.pass_type_identifier,
    // title: json.logo_text,
    // description: json.description,
    // issuer: json.organization_name,

    // barcode: barcodes.fromPkPass(json.barcodes ? json.barcodes : json.barcode ? [json.barcode] : []),
    // background_color: color(json.backgroundColor),

    // files: files,
    // strings: strings,

    // front_content: [
    //   json[pass.pkpass_content_fields].header_fields,
    //   json[pass.pkpass_content_fields].primary_fields,
    //   json[pass.pkpass_content_fields].secondary_fields,
    //   json[pass.pkpass_content_fields].auxiliary_fields,
    // ].filter(fields => fields && fields.length > 0),
    // back_content: json[pass.pkpass_content_fields].back_fields || [],
    pub serial_number: String,
    pub format_version: u64,
    pub pass_type_identifier: String,

    pub organization_name: String,
    pub team_identifier: String,

    pub description: String,

    pub background_color: String,
    pub foreground_color: String,
}

impl PkPassBase {
    fn extract(json: &serde_json::Value, year: i32) -> Result<Self, anyhow::Error> {
        let Some(obj) = json.as_object() else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Expected json to be object.".to_owned()).into());
        };

        let Some(serial_number) = obj.get("serialNumber").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBase is missing key".to_owned()).into());
        };
        let Some(format_version) = obj.get("formatVersion").and_then(|v| v.as_u64()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBase is missing key".to_owned()).into());
        };
        let Some(pass_type_identifier) = obj.get("passTypeIdentifier").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBase is missing key".to_owned()).into());
        };
        let Some(organization_name) = obj.get("organizationName").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBase is missing key".to_owned()).into());
        };
        let Some(team_identifier) = obj.get("teamIdentifier").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBase is missing key".to_owned()).into());
        };
        let Some(description) = obj.get("description").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBase is missing key".to_owned()).into());
        };
        let Some(background_color) = obj.get("backgroundColor").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBase is missing key".to_owned()).into());
        };
        let Some(foreground_color) = obj.get("foregroundColor").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassBase is missing key".to_owned()).into());
        };

        Ok(Self {
            serial_number,
            format_version,
            pass_type_identifier,
            organization_name,
            team_identifier,
            description,
            background_color,
            foreground_color,
        })
    }
}

#[derive(Debug, Clone)]
pub enum PkPassSubtype {
    Flight(PkPassFlight),
}

impl PassSubtypeOwner for PkPassSubtype {
    fn check_for_key(key: SubtypeKey, json: &serde_json::Map<String, serde_json::Value>) -> Option<&'static str> {
        match key {
            SubtypeKey::Flight => {
                const KEY: &'static str = "boardingPass";
                const SECONDARY_KEY: &'static str = "transitType";
                const AIR_TYPE: &'static str = "PKTransitTypeAir";
                let Some(map) = json.get(KEY).and_then(|j| j.as_object()) else {
                    return None;
                };
                let Some(transit_type) = map.get(SECONDARY_KEY) else {
                    return None;
                };
                if transit_type.as_str() != Some(AIR_TYPE) {
                    return None;
                };
                Some(KEY)
            },
        }
    }
}

impl PkPassSubtype {
    fn extract(json: &serde_json::Value, year: i32) -> Result<Self, anyhow::Error> {
        let Some(obj) = json.as_object() else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Expected json to be object.".to_owned()).into());
        };

        let (key, v) = match Self::extract_subtype(&json) {
            Ok(subtype_data) => subtype_data,
            Err(e) => {
              return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_owned()).into());
            },
        };

        let subtype = match key {
            SubtypeKey::Flight => {
                PkPassSubtype::Flight(PkPassFlight::extract(obj, v, year)?)
            },
        };

        Ok(subtype)
    }
}

#[derive(Debug, Clone)]
pub struct PkPassValue {
    pub key: String,
    pub label: String,
    pub value: String,
}

impl PkPassValue {
    fn extract_array(value: &Vec<serde_json::Value>) -> Result<Vec<Self>, anyhow::Error> {
        value.iter().map(|v| Self::extract(v)).collect::<Result<Vec<_>, _>>()
    }

    fn extract(value: &serde_json::Value) -> Result<Self, anyhow::Error> {
        let Some(obj) = value.as_object() else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassValue is not an object".to_owned()).into());
        };

        let Some(key) = obj.get("key").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassValue is missing key".to_owned()).into());
        };
        let Some(label) = obj.get("label").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassValue is missing key".to_owned()).into());
        };
        let Some(value) = obj.get("value").and_then(|v| v.as_str()).map(|s| s.to_owned()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassValue is missing key".to_owned()).into());
        };

        Ok(Self {
          key,
          label,
          value,
        })
    }

    fn scan_for_value_for_key<'a>(key: &str, iter: impl Iterator<Item=&'a Self>) -> Option<&'a str> {
        for entry in iter {
            if entry.key == key {
                return Some(entry.value.as_str());
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
pub enum PkTransitType {
    Air,
}

impl PkTransitType {
    fn from_str(s: &str) -> Result<Self, anyhow::Error> {
        match s {
            "PKTransitTypeAir" => Ok(Self::Air),
            _ => Err(std::io::Error::new(std::io::ErrorKind::Other, "Not a PkTransitType".to_owned()).into()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PkPassFlight {
    auxiliary_fields: Vec<PkPassValue>,
    back_fields: Vec<PkPassValue>,
    header_fields: Vec<PkPassValue>,
    primary_fields: Vec<PkPassValue>,
    secondary_fields: Vec<PkPassValue>,
    transit_type: PkTransitType,

    year: i32,
}

impl PkPassFlight {
    fn extract(json: &serde_json::Map<String, serde_json::Value>, internal_json: &serde_json::Value, year: i32) -> Result<Self, anyhow::Error> {
        let Some(obj) = internal_json.as_object() else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassValue is not an object".to_owned()).into());
        };

        let Some(auxiliary_fields) = obj.get("auxiliaryFields").and_then(|v| v.as_array()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassValue is missing key".to_owned()).into());
        };
        let Some(back_fields) = obj.get("backFields").and_then(|v| v.as_array()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassFlight is missing key".to_owned()).into());
        };
        let Some(header_fields) = obj.get("headerFields").and_then(|v| v.as_array()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassFlight is missing key".to_owned()).into());
        };
        let Some(primary_fields) = obj.get("primaryFields").and_then(|v| v.as_array()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassFlight is missing key".to_owned()).into());
        };
        let Some(secondary_fields) = obj.get("secondaryFields").and_then(|v| v.as_array()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassFlight is missing key".to_owned()).into());
        };
        let Some(transit_type) = obj.get("transitType").and_then(|v| v.as_str()) else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "PkPassFlight is missing key".to_owned()).into());
        };

        // if (this.departureDateTime === null) {
        //     return Err(std::io::Error::new(std::io::ErrorKind::Other, "Could not determine flight departure date/time, please specify via hints.json".to_owned()).into());
        // }
        Ok(Self {
            auxiliary_fields: PkPassValue::extract_array(auxiliary_fields)?,
            back_fields: PkPassValue::extract_array(back_fields)?,
            header_fields: PkPassValue::extract_array(header_fields)?,
            primary_fields: PkPassValue::extract_array(primary_fields)?,
            secondary_fields: PkPassValue::extract_array(secondary_fields)?,
            transit_type: PkTransitType::from_str(transit_type)?,
            year,
        })
    }

    fn auxiliary_field(&self, key: &str) -> Option<&str> {
        PkPassValue::scan_for_value_for_key(key, self.auxiliary_fields.iter())
    }

    fn back_field(&self, key: &str) -> Option<&str> {
        PkPassValue::scan_for_value_for_key(key, self.back_fields.iter())
    }

    fn header_field(&self, key: &str) -> Option<&str> {
        PkPassValue::scan_for_value_for_key(key, self.header_fields.iter())
    }

    fn primary_field(&self, key: &str) -> Option<&str> {
        PkPassValue::scan_for_value_for_key(key, self.primary_fields.iter())
    }

    fn secondary_field(&self, key: &str) -> Option<&str> {
        PkPassValue::scan_for_value_for_key(key, self.secondary_fields.iter())
    }

    pub fn date(&self) -> Option<&str> {
        self.auxiliary_field("Date")
    }

    pub fn boarding_time(&self) -> Option<&str> {
        self.auxiliary_field("boardingTime")
    }

    pub fn details(&self) -> Option<&str> {
        self.auxiliary_field("Details")
    }

    pub fn subsidiary_carrier(&self) -> Option<&str> {
        self.auxiliary_field("subsidiaryCarrier")
    }

    pub fn ticket(&self) -> Option<&str> {
        self.back_field("ticket")
    }

    pub fn recloc(&self) -> Option<&str> {
        self.back_field("recloc")
    }

    pub fn frequent_flyer(&self) -> Option<&str> {
        self.back_field("fqtv")
    }

    pub fn sequence(&self) -> Option<&str> {
        self.back_field("seq")
    }

    pub fn departure_time(&self) -> Option<&str> {
        self.back_field("departureTime")
    }

    pub fn seat(&self) -> Option<&str> {
        self.header_field("seat")
    }

    pub fn flight_number(&self) -> Option<&str> {
        self.header_field("flightNb")
    }

    pub fn board_point(&self) -> Option<&str> {
        self.primary_field("boardPoint")
    }

    pub fn off_point(&self) -> Option<&str> {
        self.primary_field("offPoint")
    }

    pub fn passenger(&self) -> Option<&str> {
        self.secondary_field("passenger")
    }

    pub fn booking_class(&self) -> Option<&str> {
        self.secondary_field("bookingClass")
    }

    pub fn status(&self) -> Option<&str> {
        self.secondary_field("status")
    }

    pub fn group(&self) -> Option<&str> {
        self.secondary_field("group")
    }

    pub fn parsed_boarding_datetime(&self) -> Option<chrono::DateTime<chrono::FixedOffset>> {
        let offset = chrono::FixedOffset::west_opt(chrono::Duration::hours(7).num_seconds() as i32)?;
        let time = format!("{} {} {}", self.year, self.date()?, self.boarding_time()?);
        println!("{:?}", time);
        chrono::NaiveDateTime::parse_from_str(time.as_str(), "%Y %d %b %H:%M").ok().and_then(|c| c.and_local_timezone(offset).single())
    }

    pub fn parsed_departure_datetime(&self) -> Option<chrono::DateTime<chrono::FixedOffset>> {
        // TODO Get departure time from airport code. This is PDT.
        let offset = chrono::FixedOffset::west_opt(chrono::Duration::hours(7).num_seconds() as i32)?;
        let time = format!("{} {} {}", self.year, self.date()?, self.departure_time()?);
        chrono::NaiveDateTime::parse_from_str(time.as_str(), "%Y %d %b %H:%M").ok().and_then(|c| c.and_local_timezone(offset).single())
    }
}

impl PkPass {
    pub fn read<R: Read + Seek>(pkpass_r: R, year: i32) -> Result<Self, anyhow::Error> {
        let l10n_path_part = ".lproj/";
        let mut pkpass = zip::ZipArchive::new(pkpass_r)?;

        let pass_json: serde_json::Value = serde_json::from_reader(pkpass.by_name("pass.json")?)?;
        println!("JSON\n{pass_json:#?}");

        let mut files = HashMap::new();
        for idx in 0..pkpass.len() {
            let piece = pkpass.by_index(idx)?;
            let name = piece.name().to_owned();
            let data = piece.bytes().collect::<Result<Vec<_>, _>>()?;

            println!("Processing {:?}", name);

            files.insert(name, data);
        }

        let mut l10n: HashMap<_, HashMap<_, _>> = HashMap::new();
        for (name, data) in files.iter() {
            let Some(suffix_start) = name.find(l10n_path_part) else {
                continue;
            };
            let suffix_end = suffix_start + l10n_path_part.len();

            let lang = (&name[0..suffix_start]).to_owned();
            let path = (&name[suffix_end..]).to_owned();

            l10n.entry(lang).or_default().insert(path, data.clone());
        }

        let base = PkPassBase::extract(&pass_json, year)?;
        let subtype = PkPassSubtype::extract(&pass_json, year)?;
        let barcode = PkPassBarcode::extract(&pass_json, year)?;

        Ok(PkPass {
            base,
            subtype,
            barcode,
        })
    }
}

impl From<GoogleWalletPass> for Pass {
    fn from(pass: GoogleWalletPass) -> Self {
        unimplemented!("googlepass to Pass");
    }
}

impl From<PkPass> for Pass {
    fn from(pass: PkPass) -> Self {
        unimplemented!("pkpass to Pass");
    }
}

impl From<Pass> for GoogleWalletPass {
    fn from(pass: Pass) -> Self {
        unimplemented!("pass to google");
    }
}

impl From<Pass> for PkPass {
    fn from(pass: Pass) -> Self {
        unimplemented!("pass to pkpass");
    }
}
