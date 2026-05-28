//! Interfaces Web e XML
use crate::types::WebMessage;
use quick_xml::events::Event;
use quick_xml::reader::Reader;

pub struct StrWebClient;
pub struct SpbWebClient;

impl SpbWebClient {
    pub fn sign_xml(xml: &str) -> WebMessage {
        let key = b"ICP_BRASIL_PRIVATE_KEY_MOCK";
        let signature = crate::crypto::generate_hmac(key, xml.as_bytes());
        let signature_hex = hex::encode(signature);

        WebMessage {
            payload: xml.to_string(),
            signature: signature_hex,
        }
    }

    pub fn parse_xml(xml: &str) -> Result<String, String> {
        let mut reader = Reader::from_str(xml);
        reader.trim_text(true);

        let mut buf = Vec::new();
        let mut in_signature = false;
        let mut signature_val = String::new();

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) if e.name().as_ref() == b"Signature" => {
                    in_signature = true;
                }
                Ok(Event::End(ref e)) if e.name().as_ref() == b"Signature" => {
                    in_signature = false;
                }
                Ok(Event::Text(e)) if in_signature => {
                    signature_val = e.unescape().unwrap().into_owned();
                    break;
                }
                Ok(Event::Eof) => break,
                Err(e) => return Err(format!("XML Error: {:?}", e)),
                _ => (),
            }
            buf.clear();
        }

        if signature_val.is_empty() {
            Err("Signature not found".to_string())
        } else {
            Ok(signature_val)
        }
    }
}
