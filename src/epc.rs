use crate::models::{EpcPaymentPayload, InvoiceData, InvoicerError, PaymentResult};
use qrcode::render::svg;
use qrcode::{EcLevel, QrCode};

/// Creates an EPC QR payload from extracted `InvoiceData`
pub fn build_epc_payload(invoice: &InvoiceData) -> EpcPaymentPayload {
    let beneficiary = invoice.supplier_name.chars().take(70).collect::<String>();
    let clean_iban = invoice.iban.replace(' ', "").to_uppercase();
    let bic = invoice.bic.clone().unwrap_or_default();
    let amount_str = format!("EUR{:.2}", invoice.amount_eur);
    let unstructured_ref = invoice.reference.chars().take(140).collect::<String>();

    EpcPaymentPayload {
        service_tag: "BCD".to_string(),
        version: "002".to_string(),
        character_set: "1".to_string(),
        identification: "SCT".to_string(),
        bic,
        beneficiary_name: beneficiary,
        iban: clean_iban,
        amount: amount_str,
        purpose_code: String::new(),
        structured_ref: String::new(),
        unstructured_ref,
    }
}

/// Generates an SVG string representation of the SEPA EPC QR code
pub fn generate_epc_qr_svg(epc_string: &str) -> Result<String, InvoicerError> {
    let qr = QrCode::with_error_correction_level(epc_string.as_bytes(), EcLevel::M)
        .map_err(|e| InvoicerError::QrGeneration(format!("QR generation failed: {e}")))?;

    let svg = qr
        .render::<svg::Color>()
        .min_dimensions(280, 280)
        .dark_color(svg::Color("#111827"))
        .light_color(svg::Color("#ffffff"))
        .build();

    Ok(svg)
}

/// Builds a payment result bundle containing invoice, EPC string, and SVG QR code
pub fn create_payment_result(invoice: InvoiceData) -> Result<PaymentResult, InvoicerError> {
    let epc = build_epc_payload(&invoice);
    let epc_payload = epc.to_epc_string();
    let qr_svg = generate_epc_qr_svg(&epc_payload)?;

    Ok(PaymentResult {
        invoice,
        epc_payload,
        qr_svg,
    })
}
