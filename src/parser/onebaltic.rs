use crate::models::{InvoiceData, InvoicerError};
use regex::Regex;
use std::sync::OnceLock;

static ONEBALTIC_PATTERNS: OnceLock<OneBalticRegexes> = OnceLock::new();

struct OneBalticRegexes {
    barcode_ref: Regex,
    invoice_nr_alt: Regex,
    iban: Regex,
    bic: Regex,
    due_date: Regex,
    period: Regex,
    amount_fallback: Regex,
    reg_num: Regex,
}

impl OneBalticRegexes {
    fn new() -> Self {
        Self {
            barcode_ref: Regex::new(r"(?i)\b(OBI/(\d+)/([\d]+(?:\.[\d]{2})?))\b").unwrap(),
            invoice_nr_alt: Regex::new(r"(?i)(?:RĒĶINS|Nr\.?)\s*[.\s]*(\d{4,8})").unwrap(),
            iban: Regex::new(r"\b(LV\d{2}[A-Z0-9]{17})\b").unwrap(),
            bic: Regex::new(r"\b(MULTLV2X|[A-Z]{6}[A-Z0-9]{2}(?:[A-Z0-9]{3})?)\b").unwrap(),
            due_date: Regex::new(r"\b(\d{2}\.\d{2}\.\d{4})\.").unwrap(),
            period: Regex::new(r"(\d{2}\.\d{2}\.\d{4}\s*-\s*\d{2}\.\d{2}\.\d{4})").unwrap(),
            amount_fallback: Regex::new(r"(?i)(?:Kopā|summa|apmaksai)[^\d]*([\d]+\.[\d]{2})")
                .unwrap(),
            reg_num: Regex::new(r"\b(40103332789|\d{11})\b").unwrap(),
        }
    }
}

fn regexes() -> &'static OneBalticRegexes {
    ONEBALTIC_PATTERNS.get_or_init(OneBalticRegexes::new)
}

/// Checks if the extracted text matches OneBaltic Property Management invoices
#[must_use]
pub fn is_onebaltic_invoice(text: &str) -> bool {
    text.contains("OneBaltic")
        || text.contains("MULTLV2X")
        || text.contains("40103332789")
        || text.contains("OBI/")
}

/// Parses a OneBaltic invoice text into structured `InvoiceData`
pub fn parse(text: &str) -> Result<InvoiceData, InvoicerError> {
    let re = regexes();

    // 1. Extract payment reference, invoice number, and total amount (primary source: OBI barcode)
    let (reference, invoice_num, amount_eur) = if let Some(caps) = re.barcode_ref.captures(text) {
        let full_ref = caps.get(1).map(|m| m.as_str().to_string()).unwrap();
        let inv_nr = caps.get(2).map(|m| m.as_str().to_string());
        let amt_str = caps.get(3).map(|m| m.as_str()).unwrap_or("0.00");
        let amt = amt_str.parse::<f64>().unwrap_or(0.0);
        (full_ref, inv_nr, amt)
    } else {
        let inv_nr = re
            .invoice_nr_alt
            .captures(text)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string());
        let ref_str = inv_nr
            .as_ref()
            .map(|nr| format!("Rekins {nr}"))
            .unwrap_or_else(|| "Rekins".to_string());

        let amt = re
            .amount_fallback
            .captures(text)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<f64>().ok())
            .ok_or(InvoicerError::MissingField("Amount (Kopā apmaksai)"))?;

        (ref_str, inv_nr, amt)
    };

    // 2. Extract IBAN (default to Industra Bank account for OneBaltic if standard match)
    let iban = re
        .iban
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_uppercase())
        .unwrap_or_else(|| "LV97MULT1010A80170010".to_string());

    // 3. Extract BIC
    let bic = re
        .bic
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_uppercase())
        .or_else(|| Some("MULTLV2X".to_string()));

    // 4. Supplier Info
    let supplier_name = "OneBaltic Property Management SIA".to_string();
    let supplier_reg_num = re
        .reg_num
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .or_else(|| Some("40103332789".to_string()));

    // 5. Recipient
    let recipient_name = if text.contains("Denis Glotov") {
        Some("Denis Glotov".to_string())
    } else {
        None
    };

    // 6. Dates & Period
    let due_date = re
        .due_date
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| format!("{}.", m.as_str()));

    let period = re
        .period
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string());

    let formatted_amount = format!("{amount_eur:.2}");

    Ok(InvoiceData {
        supplier_name,
        supplier_reg_num,
        recipient_name,
        iban,
        bic,
        amount_eur,
        formatted_amount,
        invoice_num,
        reference,
        period,
        due_date,
    })
}
