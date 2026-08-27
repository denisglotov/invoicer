use crate::models::{InvoiceData, InvoicerError};
use regex::Regex;
use std::sync::OnceLock;

static GENERIC_PATTERNS: OnceLock<GenericRegexes> = OnceLock::new();

struct GenericRegexes {
    iban: Regex,
    labeled_bic: Regex,
    fallback_bic: Regex,
    amount_with_eur: Regex,
    total_amount: Regex,
    invoice_nr: Regex,
    reference: Regex,
    company: Regex,
}

impl GenericRegexes {
    fn new() -> Self {
        Self {
            iban: Regex::new(r"(?i)\b([A-Z]{2}\d{2}[A-Z0-9]{11,30})\b").unwrap(),
            labeled_bic: Regex::new(r"(?i)(?:BIC|SWIFT|Bank\s+code|Bankas\s+kods)\s*[:.]?\s*([A-Z0-9]{8,11})").unwrap(),
            fallback_bic: Regex::new(r"\b([A-Z]{4}[A-Z]{2}[A-Z0-9]{2}(?:[A-Z0-9]{3})?)\b").unwrap(),
            amount_with_eur: Regex::new(r"(?i)(?:EUR|€)\s*([\d\s]+[.,]\d{2})\b|([\d\s]+[.,]\d{2})\s*(?:EUR|€)").unwrap(),
            total_amount: Regex::new(r"(?i)(?:Total|Kopā|Summa|Amount|Total\s+Due|Payable)[^\d\n\r]*([\d\s]+[.,]\d{2})").unwrap(),
            invoice_nr: Regex::new(r"(?i)(?:Invoice|Rēķins|Bill|Facture|Rechnung)\s*(?:Nr\.?|No\.?|#)?\s*([A-Za-z0-9\-_/]+)").unwrap(),
            reference: Regex::new(r"(?i)(?:Reference|Ref\.?|Mērķis|Payment\s+Details|Paskirtis|Verwendungszweck)\s*[:.]?\s*([A-Za-z0-9\-_/.]+)").unwrap(),
            company: Regex::new(r"(?i)\b([A-Z0-9\s.,'&-]+(?:SIA|AS|OÜ|UAB|GmbH|Ltd|LLC|Inc|S\.A\.))\b").unwrap(),
        }
    }
}

fn regexes() -> &'static GenericRegexes {
    GENERIC_PATTERNS.get_or_init(GenericRegexes::new)
}

/// Generic heuristic parser for European invoices
pub fn parse(text: &str) -> Result<InvoiceData, InvoicerError> {
    let re = regexes();

    let iban = re
        .iban
        .captures_iter(text)
        .find_map(|c| c.get(1).map(|m| m.as_str().replace(' ', "").to_uppercase()))
        .ok_or(InvoicerError::MissingField(
            "IBAN (not found in invoice text)",
        ))?;

    let bic = re
        .labeled_bic
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_uppercase())
        .or_else(|| {
            re.fallback_bic.captures_iter(text).find_map(|c| {
                let s = c.get(1)?.as_str();
                if s != "SUPPLIES" && s != "INVOICES" && s != "PAYMENTS" {
                    Some(s.to_uppercase())
                } else {
                    None
                }
            })
        });

    let invoice_num = re
        .invoice_nr
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string());

    let reference = re
        .reference
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .or_else(|| invoice_num.as_ref().map(|nr| format!("Invoice {nr}")))
        .unwrap_or_else(|| "Payment".to_string());

    let raw_amount = re
        .total_amount
        .captures(text)
        .and_then(|c| c.get(1))
        .or_else(|| {
            re.amount_with_eur
                .captures(text)
                .and_then(|c| c.get(1).or_else(|| c.get(2)))
        })
        .map(|m| m.as_str().trim().replace(' ', "").replace(',', "."))
        .ok_or(InvoicerError::MissingField("Total Amount"))?;

    let amount_eur = raw_amount
        .parse::<f64>()
        .map_err(|_| InvoicerError::InvalidAmount(raw_amount.clone()))?;

    let formatted_amount = format!("{amount_eur:.2}");

    let supplier_name = re
        .company
        .captures(text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().trim().to_string())
        .or_else(|| {
            text.lines()
                .find(|line| !line.trim().is_empty())
                .map(|line| line.trim().to_string())
        })
        .unwrap_or_else(|| "Payee".to_string());

    Ok(InvoiceData {
        supplier_name,
        supplier_reg_num: None,
        recipient_name: None,
        iban,
        bic,
        amount_eur,
        formatted_amount,
        invoice_num,
        reference,
        period: None,
        due_date: None,
    })
}
