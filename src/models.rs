use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum InvoicerError {
    #[error("Failed to parse PDF document: {0}")]
    PdfParse(String),

    #[error("No text could be extracted from PDF")]
    EmptyText,

    #[error("Missing required field: {0}")]
    MissingField(&'static str),

    #[error("Invalid IBAN format: {0}")]
    InvalidIban(String),

    #[error("Invalid amount format: {0}")]
    InvalidAmount(String),

    #[error("Failed to generate QR code: {0}")]
    QrGeneration(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Extracted invoice details
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InvoiceData {
    pub supplier_name: String,
    pub supplier_reg_num: Option<String>,
    pub recipient_name: Option<String>,
    pub iban: String,
    pub bic: Option<String>,
    pub amount_eur: f64,
    pub formatted_amount: String,
    pub invoice_num: Option<String>,
    pub reference: String,
    pub period: Option<String>,
    pub due_date: Option<String>,
}

/// Standard European Payments Council (EPC) SEPA QR code payload (Version 002)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EpcPaymentPayload {
    pub service_tag: String,      // BCD
    pub version: String,          // 002
    pub character_set: String,    // 1 (UTF-8)
    pub identification: String,   // SCT
    pub bic: String,              // BIC / SWIFT code
    pub beneficiary_name: String, // Up to 70 chars
    pub iban: String,             // IBAN without spaces
    pub amount: String,           // EUR followed by amount (e.g. EUR127.43)
    pub purpose_code: String,     // Optional 4-char code
    pub structured_ref: String,   // Structured reference (ISO 11649 or national)
    pub unstructured_ref: String, // Free text remittance info
}

impl EpcPaymentPayload {
    /// Formats the EPC payload according to the EPC069-08 / BCD standard guidelines
    #[must_use]
    pub fn to_epc_string(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.service_tag,
            self.version,
            self.character_set,
            self.identification,
            self.bic,
            self.beneficiary_name,
            self.iban,
            self.amount,
            self.purpose_code,
            self.structured_ref,
            self.unstructured_ref
        )
    }
}

/// Complete payment package returned to the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResult {
    pub invoice: InvoiceData,
    pub epc_payload: String,
    pub qr_svg: String,
}
