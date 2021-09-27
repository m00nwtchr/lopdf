pub mod cmap;
mod glyphnames;
mod mappings;

use crate::Error;
use crate::Result;
use cmap::ToUnicodeCMap;

use encoding_rs::UTF_16BE;

pub use self::mappings::*;

pub fn bytes_to_string(encoding: &ByteToGlyphMap, bytes: &[u8]) -> String {
    let code_points = bytes
        .iter()
        .filter_map(|&byte| encoding[byte as usize])
        .collect::<Vec<u16>>();
    String::from_utf16_lossy(&code_points)
}

pub fn string_to_bytes(encoding: &ByteToGlyphMap, text: &str) -> Vec<u8> {
    text.encode_utf16()
        .filter_map(|ch| encoding.iter().position(|&code| code == Some(ch)))
        .map(|byte| byte as u8)
        .collect()
}

#[derive(Debug)]
pub enum Encoding<'a> {
    OneByteEncoding(&'a ByteToGlyphMap),
    SimpleEncoding(&'a str),
    UnicodeMapEncoding(ToUnicodeCMap),
}

impl<'a> Encoding<'a> {
    pub fn bytes_to_string(&self, bytes: &[u8]) -> Result<String> {
        match self {
            Self::OneByteEncoding(map) => Ok(bytes_to_string(map, bytes)),
            Self::SimpleEncoding(name) if ["UniGB-UCS2-H", "UniGB−UTF16−H"].contains(name) => {
                Ok(UTF_16BE.decode(bytes).0.to_string())
            }
            Self::UnicodeMapEncoding(unicode_map) => {
                let utf16_str: Vec<u8> = bytes
                    .chunks_exact(2)
                    .map(|chunk| chunk[0] as u16 * 256 + chunk[1] as u16)
                    .flat_map(|cp| unicode_map.get_or_replacement_char(cp))
                    .flat_map(|it| [(it / 256) as u8, (it % 256) as u8])
                    .collect();

                let str = UTF_16BE.decode(&utf16_str).0.to_string();
                println!("Unicode: {str}");
                Ok(str)
            }
            _ => Err(Error::ContentDecode),
        }
    }
    pub fn string_to_bytes(&self, text: &str) -> Vec<u8> {
        match self {
            Self::OneByteEncoding(map) => string_to_bytes(map, text),
            Self::SimpleEncoding(name) if ["UniGB-UCS2-H", "UniGB-UTF16-H"].contains(name) => {
                UTF_16BE.encode(text).0.to_vec()
            }
            Self::UnicodeMapEncoding(_unicode_map) => {
                //maybe only possible if the unicode map is an identity?
                unimplemented!()
            }
            _ => string_to_bytes(&STANDARD_ENCODING, text),
        }
    }
}
