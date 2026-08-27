pub mod generic;
pub mod onebaltic;
pub mod pdf_text;

use crate::models::{InvoiceData, InvoicerError};

/// Parses raw PDF bytes and extracts structured invoice data
pub fn parse_invoice_from_bytes(pdf_bytes: &[u8]) -> Result<InvoiceData, InvoicerError> {
    let text = pdf_text::extract_text_from_pdf(pdf_bytes)?;
    parse_invoice_from_text(&text)
}

/// Parses extracted text into structured invoice data
pub fn parse_invoice_from_text(text: &str) -> Result<InvoiceData, InvoicerError> {
    if onebaltic::is_onebaltic_invoice(text) {
        if let Ok(data) = onebaltic::parse(text) {
            return Ok(data);
        }
    }

    generic::parse(text)
}
