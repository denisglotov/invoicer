use crate::models::InvoicerError;
use lopdf::content::Content;
use lopdf::{Document, Object};

/// Extracts all text strings from a PDF document.
pub fn extract_text_from_pdf(pdf_bytes: &[u8]) -> Result<String, InvoicerError> {
    let doc = Document::load_mem(pdf_bytes)
        .map_err(|e| InvoicerError::PdfParse(format!("Failed to load PDF document: {e}")))?;

    // Try lopdf built-in extractor first if available
    let page_numbers: Vec<u32> = (1..=doc.get_pages().len() as u32).collect();
    if let Ok(text) = doc.extract_text(&page_numbers) {
        if !text.trim().is_empty() {
            return Ok(normalize_extracted_text(&text));
        }
    }

    // Robust operator walker for documents with non-standard CMaps
    let mut extracted_chunks = Vec::new();

    for (_page_num, page_id) in doc.get_pages() {
        if let Ok(content_data) = doc.get_page_content(page_id) {
            if let Ok(content) = Content::decode(&content_data) {
                let page_text = extract_text_from_operations(&content.operations);
                if !page_text.trim().is_empty() {
                    extracted_chunks.push(page_text);
                }
            }
        }
    }

    if extracted_chunks.is_empty() {
        // Fallback scan through all object streams in document
        for object in doc.objects.values() {
            if let Ok(stream) = object.as_stream() {
                if let Ok(decompressed) = stream.decompressed_content() {
                    if let Ok(content) = Content::decode(&decompressed) {
                        let text = extract_text_from_operations(&content.operations);
                        if !text.trim().is_empty() {
                            extracted_chunks.push(text);
                        }
                    }
                }
            }
        }
    }

    let combined = extracted_chunks.join("\n");
    if combined.trim().is_empty() {
        Err(InvoicerError::EmptyText)
    } else {
        Ok(normalize_extracted_text(&combined))
    }
}

/// Decodes text from PDF operators (Tj, TJ, ', ")
fn extract_text_from_operations(operations: &[lopdf::content::Operation]) -> String {
    let mut text_lines = Vec::new();
    let mut current_line = String::new();

    for op in operations {
        match op.operator.as_str() {
            "Tj" | "'" | "\"" => {
                if let Some(operand) = op.operands.first() {
                    if let Some(s) = object_to_string(operand) {
                        current_line.push_str(&s);
                    }
                }
                if (op.operator == "'" || op.operator == "\"") && !current_line.trim().is_empty() {
                    text_lines.push(current_line.trim().to_string());
                    current_line.clear();
                }
            }
            "TJ" => {
                if let Some(Object::Array(arr)) = op.operands.first() {
                    for item in arr {
                        match item {
                            Object::String(..) => {
                                if let Some(s) = object_to_string(item) {
                                    current_line.push_str(&s);
                                }
                            }
                            Object::Integer(n) if *n < -250 => {
                                current_line.push(' ');
                            }
                            Object::Real(r) if *r < -250.0 => {
                                current_line.push(' ');
                            }
                            _ => {}
                        }
                    }
                }
            }
            "T*" | "TD" | "Td" | "ET" if !current_line.trim().is_empty() => {
                text_lines.push(current_line.trim().to_string());
                current_line.clear();
            }
            _ => {}
        }
    }

    if !current_line.trim().is_empty() {
        text_lines.push(current_line.trim().to_string());
    }

    text_lines.join("\n")
}

/// Normalizes extracted text lines
fn normalize_extracted_text(text: &str) -> String {
    text.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Converts a PDF Object (String or Name) to Rust String, handling Latin1/Windows-1257/UTF-8
fn object_to_string(obj: &Object) -> Option<String> {
    match obj {
        Object::String(bytes, _) => {
            // Try UTF-8 first
            if let Ok(s) = String::from_utf8(bytes.clone()) {
                Some(s)
            } else if bytes.starts_with(&[0xFE, 0xFF]) {
                // UTF-16BE
                let u16_chars: Vec<u16> = bytes[2..]
                    .chunks_exact(2)
                    .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
                    .collect();
                String::from_utf16(&u16_chars).ok()
            } else {
                // Decode Windows-1257 / ISO-8859-4 for Baltic characters
                Some(decode_baltic_bytes(bytes))
            }
        }
        Object::Name(bytes) => String::from_utf8(bytes.clone()).ok(),
        _ => None,
    }
}

/// Decodes Baltic bytes (Windows-1257 / ISO-8859-13) to UTF-8
fn decode_baltic_bytes(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| match b {
            0xC0 => 'Ā',
            0xC8 => 'Č',
            0xCB => 'Ē',
            0xCE => 'Ģ',
            0xCF => 'Ī',
            0xD3 => 'Ķ',
            0xD5 => 'Ļ',
            0xD9 => 'Ņ',
            0xDA => 'Š',
            0xDE => 'Ū',
            0xDF => 'Ž',
            0xE0 => 'ā',
            0xE8 => 'č',
            0xEB => 'ē',
            0xEE => 'ģ',
            0xEF => 'ī',
            0xF3 => 'ķ',
            0xF5 => 'ļ',
            0xF9 => 'ņ',
            0xFA => 'š',
            0xFE => 'ū',
            0xFF => 'ž',
            32..=126 => b as char,
            _ => b as char,
        })
        .collect()
}
