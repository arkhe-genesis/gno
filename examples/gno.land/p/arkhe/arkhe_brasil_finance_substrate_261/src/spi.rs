//! Cliente SPI (Pix)
use crate::types::{DictKey, PixTransaction};
use reqwest::Client;

pub struct SpiClient;

impl SpiClient {
    pub fn new() -> Self { Self }
    pub async fn process_pix(&self, tx: &PixTransaction) -> Result<String, String> {
        Ok(format!("PIX_OK_{}", tx.end_to_end_id))
    }
}

pub struct DictResolver;
impl DictResolver {
    pub async fn resolve(key: &DictKey) -> Option<String> {
        let client = Client::new();
        // Uses the sandbox endpoint defined in schema.yaml
        let url = format!("https://sandbox.spi.bcb.gov.br/api/dict/v2/key/{}", key.key);

        match client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.text().await {
                        Ok(body) => Some(body),
                        Err(_) => None,
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }
}

fn crc16_ccitt(data: &str) -> String {
    let mut crc: u16 = 0xFFFF;
    for byte in data.bytes() {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if (crc & 0x8000) != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    format!("{:04X}", crc)
}

pub fn generate_qr(tx: &PixTransaction) -> String {
    let key = &tx.receiver_key.key;
    let payload_format = "000201";
    let merchant_account_gui = "0014br.gov.bcb.pix";
    let merchant_account_key = format!("01{:02}{}", key.len(), key);
    let merchant_account_info = format!("26{:02}{}{}",
        merchant_account_gui.len() + merchant_account_key.len(),
        merchant_account_gui,
        merchant_account_key);

    let merchant_category = "52040000";
    let currency = "5303986";
    let amount_str = format!("{:.2}", tx.amount);
    let amount_field = format!("54{:02}{}", amount_str.len(), amount_str);
    let country = "5802BR";
    let merchant_name = "5909ARKHE-OS";
    let merchant_city = "6008BRASILIA";
    let additional_data_field = "62070503***";

    let mut br_code = format!(
        "{}{}{}{}{}{}{}{}{}",
        payload_format,
        merchant_account_info,
        merchant_category,
        currency,
        amount_field,
        country,
        merchant_name,
        merchant_city,
        additional_data_field
    );

    br_code.push_str("6304");

    let crc = crc16_ccitt(&br_code);
    br_code.push_str(&crc);

    br_code
}
