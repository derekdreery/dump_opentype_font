use anyhow::{format_err, Error, Result};
use serde::Serialize;
use std::{
    convert::{TryFrom, TryInto},
    fs, io,
};
use structopt::StructOpt;
use ttf_parser::PlatformId;

#[derive(StructOpt)]
struct Opt {
    font_file: String,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let raw = fs::read(&opt.font_file)?;
    let data = Collection::from_bytes(&raw)?;
    serde_json::to_writer(io::stdout().lock(), &data)?;
    println!();
    Ok(())
}

#[derive(Serialize)]
pub struct Collection {
    fonts: Vec<Font>,
}

impl Collection {
    fn from_bytes(input: &[u8]) -> Result<Self> {
        let count = ttf_parser::fonts_in_collection(input).unwrap_or(1);
        let mut fonts = Vec::with_capacity(count.try_into().unwrap());
        for ix in 0..count {
            fonts.push(Font::from_bytes(input, ix)?);
        }
        Ok(Collection { fonts })
    }
}

#[derive(Serialize)]
pub struct Font {
    names: Vec<Name>,
    family_name: Option<String>,
    post_script_name: Option<String>,
    is_regular: bool,
    is_italic: bool,
    is_bold: bool,
    is_oblique: bool,
    is_variable: bool,
    weight: String,
    width: String,
    ascender: i16,
    descender: i16,
    height: i16,
    line_gap: i16,
    vertical_ascender: Option<i16>,
    vertical_descender: Option<i16>,
    vertical_height: Option<i16>,
    vertical_line_gap: Option<i16>,
    units_per_em: Option<u16>,
    x_height: Option<i16>,
    underline_metrics: Option<String>,
    strikeout_metrics: Option<String>,
    subscript_metrics: Option<String>,
    superscript_metrics: Option<String>,
    variation_axes: Vec<VariationAxis>,
}

impl Font {
    fn from_bytes(input: &[u8], index: u32) -> Result<Self> {
        let font = ttf_parser::Font::from_data(input, index)
            .ok_or_else(|| format_err!("cannot parse font at index {}", index))?;
        Ok(Font {
            names: font
                .names()
                .map(Name::try_from)
                .collect::<Result<Vec<_>>>()?,
            family_name: font.family_name(),
            post_script_name: font.post_script_name(),
            is_regular: font.is_regular(),
            is_italic: font.is_italic(),
            is_bold: font.is_bold(),
            is_oblique: font.is_oblique(),
            is_variable: font.is_variable(),
            weight: format!("{:?}", font.weight()),
            width: format!("{:?}", font.width()),
            ascender: font.ascender(),
            descender: font.descender(),
            height: font.height(),
            line_gap: font.line_gap(),
            vertical_ascender: font.vertical_ascender(),
            vertical_descender: font.vertical_descender(),
            vertical_height: font.vertical_height(),
            vertical_line_gap: font.vertical_line_gap(),
            units_per_em: font.units_per_em(),
            x_height: font.x_height(),
            underline_metrics: font.underline_metrics().map(|m| format!("{:?}", m)),
            strikeout_metrics: font.strikeout_metrics().map(|m| format!("{:?}", m)),
            subscript_metrics: font.subscript_metrics().map(|m| format!("{:?}", m)),
            superscript_metrics: font.superscript_metrics().map(|m| format!("{:?}", m)),
            variation_axes: font.variation_axes().map(Into::into).collect(),
        })
    }
}

#[derive(Serialize)]
pub struct Name {
    name_id: NameId,
    name: String,
    platform_id: Option<String>,
    language: &'static str,
    encoding_id: u16,
    language_id: u16,
}

impl TryFrom<ttf_parser::Name<'_>> for Name {
    type Error = Error;
    fn try_from(name: ttf_parser::Name<'_>) -> Result<Self, Self::Error> {
        Ok(Name {
            platform_id: name.platform_id().map(|id| format!("{:?}", id)),
            name: {
                let name_bytes = name.name();
                // rough hack
                if name_bytes[0] == 0 && (name_bytes.len() % 2) == 0 {
                    let iter = (0..name_bytes.len() / 2)
                        .map(|i| u16::from_be_bytes([name_bytes[2 * i], name_bytes[2 * i + 1]]));
                    std::char::decode_utf16(iter).collect::<Result<String, _>>()?
                } else {
                    String::from_utf8_lossy(name.name()).into_owned()
                }
            },
            language: language(name.platform_id(), name.language_id()),
            name_id: NameId::from(name.name_id()),
            encoding_id: name.encoding_id(),
            language_id: name.language_id(),
        })
    }
}

#[derive(Serialize)]
pub enum NameId {
    CompatibleFull,
    CopyrightNotice,
    DarkBackgroundPallete,
    Description,
    Designer,
    DesignerURL,
    Family,
    FullName,
    License,
    LicenseURL,
    LightBackgroundPallete,
    Manufacturer,
    PostScriptCID,
    PostScriptName,
    SampleText,
    Subfamily,
    Trademark,
    TypographicFamily,
    TypographicSubfamily,
    UniqueID,
    VariationsPostScriptNamePrefix,
    VendorURL,
    Version,
    WWSFamily,
    WWSSubfamily,
    Unrecognised(u16),
}

impl From<u16> for NameId {
    fn from(raw: u16) -> Self {
        use ttf_parser::name_id::*;
        match raw {
            COMPATIBLE_FULL => NameId::CompatibleFull,
            COPYRIGHT_NOTICE => NameId::CopyrightNotice,
            DARK_BACKGROUND_PALETTE => NameId::DarkBackgroundPallete,
            DESCRIPTION => NameId::Description,
            DESIGNER => NameId::Designer,
            DESIGNER_URL => NameId::DesignerURL,
            FAMILY => NameId::Family,
            FULL_NAME => NameId::FullName,
            LICENSE => NameId::License,
            LICENSE_URL => NameId::LicenseURL,
            LIGHT_BACKGROUND_PALETTE => NameId::LightBackgroundPallete,
            MANUFACTURER => NameId::Manufacturer,
            POST_SCRIPT_CID => NameId::PostScriptCID,
            POST_SCRIPT_NAME => NameId::PostScriptName,
            SAMPLE_TEXT => NameId::SampleText,
            SUBFAMILY => NameId::Subfamily,
            TRADEMARK => NameId::Trademark,
            TYPOGRAPHIC_FAMILY => NameId::TypographicFamily,
            TYPOGRAPHIC_SUBFAMILY => NameId::TypographicSubfamily,
            UNIQUE_ID => NameId::UniqueID,
            VARIATIONS_POST_SCRIPT_NAME_PREFIX => NameId::VariationsPostScriptNamePrefix,
            VENDOR_URL => NameId::VendorURL,
            VERSION => NameId::Version,
            WWS_FAMILY => NameId::WWSFamily,
            WWS_SUBFAMILY => NameId::WWSSubfamily,
            other => NameId::Unrecognised(other),
        }
    }
}

fn language(platform_id: Option<PlatformId>, language_id: u16) -> &'static str {
    match platform_id {
        // from https://docs.microsoft.com/en-us/typography/opentype/spec/name TODO add mac.
        Some(PlatformId::Windows) => match language_id {
            0 => "None",
            0x0436 => "Afrikaans (South Africa)",
            0x041C => "Albanian (Albania)",
            0x0484 => "Alsatian (France)",
            0x045E => "Amharic (Ethiopia)",
            0x1401 => "Arabic (Algeria)",
            0x3C01 => "Arabic (Bahrain)",
            0x0C01 => "Arabic (Egypt)",
            0x0801 => "Arabic (Iraq)",
            0x2C01 => "Arabic (Jordan)",
            0x3401 => "Arabic (Kuwait)",
            0x3001 => "Arabic (Lebanon)",
            0x1001 => "Arabic (Libya)",
            0x1801 => "Arabic (Morocco)",
            0x2001 => "Arabic (Oman)",
            0x4001 => "Arabic (Qatar)",
            0x0401 => "Arabic (Saudi Arabia)",
            0x2801 => "Arabic (Syria)",
            0x1C01 => "Arabic (Tunisia)",
            0x3801 => "Arabic (U.A.E.)",
            0x2401 => "Arabic (Yemen)",
            0x042B => "Armenian (Armenia)",
            0x044D => "Assamese (India)",
            0x082C => "Azeri (Cyrillic) (Azerbaijan)",
            0x042C => "Azeri (Latin) (Azerbaijan)",
            0x046D => "Bashkir (Russia)",
            0x042D => "Basque (Basque)",
            0x0423 => "Belarusian (Belarus)",
            0x0845 => "Bengali (Bangladesh)",
            0x0445 => "Bengali (India)",
            0x201A => "Bosnian (Cyrillic) (Bosnia and Herzegovina)",
            0x141A => "Bosnian (Latin) (Bosnia and Herzegovina)",
            0x047E => "Breton (France)",
            0x0402 => "Bulgarian (Bulgaria)",
            0x0403 => "Catalan (Catalan)",
            0x0C04 => "Chinese (Hong Kong S.A.R.)",
            0x1404 => "Chinese (Macao S.A.R.)",
            0x0804 => "Chinese (People’s Republic of China)",
            0x1004 => "Chinese (Singapore)",
            0x0404 => "Chinese (Taiwan)",
            0x0483 => "Corsican (France)",
            0x041A => "Croatian (Croatia)",
            0x101A => "Croatian (Latin) (Bosnia and Herzegovina)",
            0x0405 => "Czech (Czech Republic)",
            0x0406 => "Danish (Denmark)",
            0x048C => "Dari (Afghanistan)",
            0x0465 => "Divehi (Maldives)",
            0x0813 => "Dutch (Belgium)",
            0x0413 => "Dutch (Netherlands)",
            0x0C09 => "English (Australia)",
            0x2809 => "English (Belize)",
            0x1009 => "English (Canada)",
            0x2409 => "English (Caribbean)",
            0x4009 => "English (India)",
            0x1809 => "English (Ireland)",
            0x2009 => "English (Jamaica)",
            0x4409 => "English (Malaysia)",
            0x1409 => "English (New Zealand)",
            0x3409 => "English (Republic of the Philippines)",
            0x4809 => "English (Singapore)",
            0x1C09 => "English (South Africa)",
            0x2C09 => "English (Trinidad and Tobago)",
            0x0809 => "English (United Kingdom)",
            0x0409 => "English (United States)",
            0x3009 => "English (Zimbabwe)",
            0x0425 => "Estonian (Estonia)",
            0x0438 => "Faroese (Faroe Islands)",
            0x0464 => "Filipino (Philippines)",
            0x040B => "Finnish (Finland)",
            0x080C => "French (Belgium)",
            0x0C0C => "French (Canada)",
            0x040C => "French (France)",
            0x140c => "French (Luxembourg)",
            0x180C => "French (Principality of Monaco)",
            0x100C => "French (Switzerland)",
            0x0462 => "Frisian (Netherlands)",
            0x0456 => "Galician (Galician)",
            0x0437 => "Georgian (Georgia)",
            0x0C07 => "German (Austria)",
            0x0407 => "German (Germany)",
            0x1407 => "German (Liechtenstein)",
            0x1007 => "German (Luxembourg)",
            0x0807 => "German (Switzerland)",
            0x0408 => "Greek (Greece)",
            0x046F => "Greenlandic (Greenland)",
            0x0447 => "Gujarati (India)",
            0x0468 => "Hausa (Latin) (Nigeria)",
            0x040D => "Hebrew (Israel)",
            0x0439 => "Hindi (India)",
            0x040E => "Hungarian (Hungary)",
            0x040F => "Icelandic (Iceland)",
            0x0470 => "Igbo (Nigeria)",
            0x0421 => "Indonesian (Indonesia)",
            0x045D => "Inuktitut (Canada)",
            0x085D => "Inuktitut (Latin) (Canada)",
            0x083C => "Irish (Ireland)",
            0x0434 => "isiXhosa (South Africa)",
            0x0435 => "isiZulu (South Africa)",
            0x0410 => "Italian (Italy)",
            0x0810 => "Italian (Switzerland)",
            0x0411 => "Japanese (Japan)",
            0x044B => "Kannada (India)",
            0x043F => "Kazakh (Kazakhstan)",
            0x0453 => "Khmer (Cambodia)",
            0x0486 => "K’iche (Guatemala)",
            0x0487 => "Kinyarwanda (Rwanda)",
            0x0441 => "Kiswahili (Kenya)",
            0x0457 => "Konkani (India)",
            0x0412 => "Korean (Korea)",
            0x0440 => "Kyrgyz (Kyrgyzstan)",
            0x0454 => "Lao (Lao P.D.R.)",
            0x0426 => "Latvian (Latvia)",
            0x0427 => "Lithuanian (Lithuania)",
            0x082E => "Lower Sorbian (Germany)",
            0x046E => "Luxembourgish (Luxembourg)",
            0x042F => "Macedonian (FYROM) (Former Yugoslav Republic of Macedonia)",
            0x083E => "Malay (Brunei Darussalam)",
            0x043E => "Malay (Malaysia)",
            0x044C => "Malayalam (India)",
            0x043A => "Maltese (Malta)",
            0x0481 => "Maori (New Zealand)",
            0x047A => "Mapudungun (Chile)",
            0x044E => "Marathi (India)",
            0x047C => "Mohawk (Mohawk)",
            0x0450 => "Mongolian (Cyrillic) (Mongolia)",
            0x0850 => "Mongolian (Traditional) (People’s Republic of China)",
            0x0461 => "Nepali (Nepal)",
            0x0414 => "Norwegian (Bokmal) (Norway)",
            0x0814 => "Norwegian (Nynorsk) (Norway)",
            0x0482 => "Occitan (France)",
            0x0448 => "Odia (formerly Oriya) (India)",
            0x0463 => "Pashto (Afghanistan)",
            0x0415 => "Polish (Poland)",
            0x0416 => "Portuguese (Brazil)",
            0x0816 => "Portuguese (Portugal)",
            0x0446 => "Punjabi (India)",
            0x046B => "Quechua (Bolivia)",
            0x086B => "Quechua (Ecuador)",
            0x0C6B => "Quechua (Peru)",
            0x0418 => "Romanian (Romania)",
            0x0417 => "Romansh (Switzerland)",
            0x0419 => "Russian (Russia)",
            0x243B => "Sami (Inari) (Finland)",
            0x103B => "Sami (Lule) (Norway)",
            0x143B => "Sami (Lule) (Sweden)",
            0x0C3B => "Sami (Northern) (Finland)",
            0x043B => "Sami (Northern) (Norway)",
            0x083B => "Sami (Northern) (Sweden)",
            0x203B => "Sami (Skolt) (Finland)",
            0x183B => "Sami (Southern) (Norway)",
            0x1C3B => "Sami (Southern) (Sweden)",
            0x044F => "Sanskrit (India)",
            0x1C1A => "Serbian (Cyrillic) (Bosnia and Herzegovina)",
            0x0C1A => "Serbian (Cyrillic) (Serbia)",
            0x181A => "Serbian (Latin) (Bosnia and Herzegovina)",
            0x081A => "Serbian (Latin) (Serbia)",
            0x046C => "Sesotho sa Leboa (South Africa)",
            0x0432 => "Setswana (South Africa)",
            0x045B => "Sinhala (Sri Lanka)",
            0x041B => "Slovak (Slovakia)",
            0x0424 => "Slovenian (Slovenia)",
            0x2C0A => "Spanish (Argentina)",
            0x400A => "Spanish (Bolivia)",
            0x340A => "Spanish (Chile)",
            0x240A => "Spanish (Colombia)",
            0x140A => "Spanish (Costa Rica)",
            0x1C0A => "Spanish (Dominican Republic)",
            0x300A => "Spanish (Ecuador)",
            0x440A => "Spanish (El Salvador)",
            0x100A => "Spanish (Guatemala)",
            0x480A => "Spanish (Honduras)",
            0x080A => "Spanish (Mexico)",
            0x4C0A => "Spanish (Nicaragua)",
            0x180A => "Spanish (Panama)",
            0x3C0A => "Spanish (Paraguay)",
            0x280A => "Spanish (Peru)",
            0x500A => "Spanish (Puerto Rico)",
            0x0C0A => "Spanish (Modern Sort) (Spain)",
            0x040A => "Spanish (Traditional Sort) (Spain)",
            0x540A => "Spanish (United States)",
            0x380A => "Spanish (Uruguay)",
            0x200A => "Spanish (Venezuela)",
            0x081D => "Sweden (Finland)",
            0x041D => "Swedish (Sweden)",
            0x045A => "Syriac (Syria)",
            0x0428 => "Tajik (Cyrillic) (Tajikistan)",
            0x085F => "Tamazight (Latin) (Algeria)",
            0x0449 => "Tamil (India)",
            0x0444 => "Tatar (Russia)",
            0x044A => "Telugu (India)",
            0x041E => "Thai (Thailand)",
            0x0451 => "Tibetan (PRC)",
            0x041F => "Turkish (Turkey)",
            0x0442 => "Turkmen (Turkmenistan)",
            0x0480 => "Uighur (PRC)",
            0x0422 => "Ukrainian (Ukraine)",
            0x042E => "Upper Sorbian (Germany)",
            0x0420 => "Urdu (Islamic Republic of Pakistan)",
            0x0843 => "Uzbek (Cyrillic) (Uzbekistan)",
            0x0443 => "Uzbek (Latin) (Uzbekistan)",
            0x042A => "Vietnamese (Vietnam)",
            0x0452 => "Welsh (United Kingdom)",
            0x0488 => "Wolof (Senegal)",
            0x0485 => "Yakut (Russia)",
            0x0478 => "Yi (PRC)",
            0x046A => "Yoruba (Nigeria)",
            _ => "unknown",
        },
        _ => "unknown (todo)",
    }
}

#[derive(Serialize)]
pub struct VariationAxis {
    tag: Option<String>,
    min_value: f32,
    default_value: f32,
    max_value: f32,
    name_id: u16,
    hidden: bool,
}

impl From<ttf_parser::VariationAxis> for VariationAxis {
    fn from(raw: ttf_parser::VariationAxis) -> Self {
        Self {
            tag: if raw.tag.is_null() {
                None
            } else {
                let c = raw.tag.to_chars();
                Some(format!("{}{}{}{}", c[0], c[1], c[2], c[3]))
            },
            min_value: raw.min_value,
            default_value: raw.def_value,
            max_value: raw.max_value,
            name_id: raw.name_id,
            hidden: raw.hidden,
        }
    }
}
