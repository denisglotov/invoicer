pub mod epc;
pub mod models;
pub mod parser;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn init_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Parses PDF bytes and generates the SEPA EPC QR code payment payload
#[wasm_bindgen]
pub fn parse_pdf_and_create_payment(pdf_bytes: &[u8]) -> Result<JsValue, JsValue> {
    let invoice = parser::parse_invoice_from_bytes(pdf_bytes)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let payment_result =
        epc::create_payment_result(invoice).map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_wasm_bindgen::to_value(&payment_result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
}

/// Parses invoice text and generates the SEPA EPC QR code payment payload
#[wasm_bindgen]
pub fn parse_text_and_create_payment(text: &str) -> Result<JsValue, JsValue> {
    let invoice =
        parser::parse_invoice_from_text(text).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let payment_result =
        epc::create_payment_result(invoice).map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_wasm_bindgen::to_value(&payment_result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {e}")))
}

/// Generates an SVG QR code from a raw EPC text payload
#[wasm_bindgen]
pub fn generate_epc_qr(epc_payload: &str) -> Result<String, JsValue> {
    epc::generate_epc_qr_svg(epc_payload).map_err(|e| JsValue::from_str(&e.to_string()))
}
