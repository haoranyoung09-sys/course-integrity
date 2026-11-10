use std::io::{Cursor, Read};
use std::path::Path;

use quick_xml::events::Event;
use quick_xml::Reader;
use zip::ZipArchive;

use crate::error::Error;
use crate::method::OCR_DEFERRED;

const MAX_ZIP_ENTRIES: usize = 128;
const MAX_ENTRY_BYTES: u64 = 8 * 1024 * 1024;
const MAX_TOTAL_UNCOMPRESSED: u64 = 32 * 1024 * 1024;
const MAX_XML_DEPTH: usize = 200;

#[derive(Debug)]
pub(crate) struct Extracted {
    pub text: String,
    pub origin: &'static str,
    pub locations: Vec<String>,
    pub diagnostics: Vec<String>,
}

pub(crate) fn extract_docx(path: &Path, bytes: &[u8]) -> Result<Extracted, Error> {
    let mut archive = ZipArchive::new(Cursor::new(bytes)).map_err(|err| Error::Extract {
        path: path.to_path_buf(),
        message: format!(
            "this is not a readable DOCX zip ({err}). The file was not uploaded. Legacy .doc is not read; save as .docx if that is what you have."
        ),
    })?;
    if archive.len() > MAX_ZIP_ENTRIES {
        return Err(Error::Extract {
            path: path.to_path_buf(),
            message: format!(
                "DOCX zip has {} entries, over the limit of {MAX_ZIP_ENTRIES}. Nothing was extracted.",
                archive.len()
            ),
        });
    }
    let mut images = 0usize;
    let mut unsafe_names = 0usize;
    let mut document_xml: Option<Vec<u8>> = None;
    let mut extras: Vec<(String, Vec<u8>)> = Vec::new();
    let mut total_uncompressed = 0u64;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(|err| Error::Extract {
            path: path.to_path_buf(),
            message: format!("could not read DOCX zip entry {index} ({err})"),
        })?;
        let raw_name = entry.name().replace('\\', "/");
        if raw_name.contains("..") || raw_name.starts_with('/') || entry.enclosed_name().is_none() {
            unsafe_names += 1;
            continue;
        }
        if entry.size() > MAX_ENTRY_BYTES {
            return Err(Error::Extract {
                path: path.to_path_buf(),
                message: format!(
                    "DOCX entry {raw_name} declares {} uncompressed bytes, over the limit of {MAX_ENTRY_BYTES}. Nothing was extracted.",
                    entry.size()
                ),
            });
        }
        let payload = read_capped(path, &raw_name, &mut entry)?;
        total_uncompressed += payload.len() as u64;
        if total_uncompressed > MAX_TOTAL_UNCOMPRESSED {
            return Err(Error::Extract {
                path: path.to_path_buf(),
                message: format!(
                    "DOCX uncompressed text exceeded {MAX_TOTAL_UNCOMPRESSED} bytes. Nothing further was extracted."
                ),
            });
        }
        if raw_name.starts_with("word/media/") {
            images += 1;
            continue;
        }
        if raw_name == "word/document.xml" {
            document_xml = Some(payload);
        } else if is_header_or_footer(&raw_name) {
            extras.push((raw_name, payload));
        }
    }
    let Some(document_xml) = document_xml else {
        return Err(Error::Extract {
            path: path.to_path_buf(),
            message: "DOCX has no word/document.xml. Nothing was extracted.".to_string(),
        });
    };
    let mut blocks = Vec::new();
    let mut locations = Vec::new();
    push_xml_paragraphs(
        path,
        "word/document.xml",
        &document_xml,
        &mut blocks,
        &mut locations,
    )?;
    extras.sort_by(|left, right| left.0.cmp(&right.0));
    for (name, xml) in extras {
        push_xml_paragraphs(path, &name, &xml, &mut blocks, &mut locations)?;
    }
    let mut diagnostics = vec![
        format!(
            "Extracted {} paragraph(s) from DOCX XML. Locations name the WordprocessingML file and paragraph number.",
            blocks.len()
        ),
        OCR_DEFERRED.to_string(),
        "Drawings, text boxes, and charts were not read. This is text extraction, not figure understanding.".to_string(),
    ];
    if images > 0 {
        diagnostics.push(format!(
            "{images} embedded image(s) under word/media were skipped and are not described."
        ));
    }
    if unsafe_names > 0 {
        diagnostics.push(format!(
            "{unsafe_names} zip entr(y/ies) with an unsafe path were skipped."
        ));
    }
    if blocks.is_empty() {
        diagnostics.push(
            "No text paragraphs were found. The file may be empty or contain only drawings."
                .to_string(),
        );
    }
    Ok(Extracted {
        text: blocks.join("\n\n"),
        origin: "docx",
        locations,
        diagnostics,
    })
}

pub(crate) fn extract_pdf(path: &Path, bytes: &[u8]) -> Result<Extracted, Error> {
    if !bytes.starts_with(b"%PDF-") {
        return Err(Error::Extract {
            path: path.to_path_buf(),
            message:
                "file does not start with a PDF header. Nothing was extracted, and OCR was not run."
                    .to_string(),
        });
    }
    let pages = pdf_extract::extract_text_from_mem_by_pages(bytes).map_err(|err| Error::Extract {
        path: path.to_path_buf(),
        message: format!(
            "could not extract text from this PDF ({err}). Passwords are not tried. OCR was not run, and the file was not uploaded."
        ),
    })?;
    let mut blocks = Vec::new();
    let mut locations = Vec::new();
    let mut empty_pages = 0usize;
    for (index, page) in pages.iter().enumerate() {
        let page_no = index + 1;
        let pieces = split_blocks(page);
        if pieces.is_empty() {
            empty_pages += 1;
            continue;
        }
        for (piece_index, piece) in pieces.into_iter().enumerate() {
            locations.push(format!("page {page_no} paragraph {}", piece_index + 1));
            blocks.push(piece);
        }
    }
    let mut diagnostics = vec![
        format!(
            "Extracted text from {} PDF page(s). {empty_pages} page(s) had no text layer.",
            pages.len()
        ),
        "Page locations are the PDF page numbers whose text layer produced characters. Layout, figures, and reading order can be wrong.".to_string(),
        OCR_DEFERRED.to_string(),
    ];
    if blocks.is_empty() {
        diagnostics.push(
            "No extractable text was found. This often means a scan or a figure-only file. OCR was not run.".to_string(),
        );
    }
    Ok(Extracted {
        text: blocks.join("\n\n"),
        origin: "pdf",
        locations,
        diagnostics,
    })
}

fn is_header_or_footer(name: &str) -> bool {
    let file = name.rsplit('/').next().unwrap_or(name);
    (file.starts_with("header") || file.starts_with("footer"))
        && file.ends_with(".xml")
        && name.starts_with("word/")
}

fn read_capped(
    path: &Path,
    name: &str,
    entry: &mut zip::read::ZipFile<'_>,
) -> Result<Vec<u8>, Error> {
    let mut limited = entry.take(MAX_ENTRY_BYTES + 1);
    let mut buf = Vec::new();
    limited
        .read_to_end(&mut buf)
        .map_err(|err| Error::Extract {
            path: path.to_path_buf(),
            message: format!("could not read DOCX entry {name} ({err})"),
        })?;
    if buf.len() as u64 > MAX_ENTRY_BYTES {
        return Err(Error::Extract {
            path: path.to_path_buf(),
            message: format!(
                "DOCX entry {name} exceeded {MAX_ENTRY_BYTES} uncompressed bytes while reading. Nothing further was extracted."
            ),
        });
    }
    Ok(buf)
}

fn push_xml_paragraphs(
    path: &Path,
    location_file: &str,
    xml_bytes: &[u8],
    blocks: &mut Vec<String>,
    locations: &mut Vec<String>,
) -> Result<(), Error> {
    let xml = String::from_utf8(xml_bytes.to_vec()).map_err(|_| Error::Extract {
        path: path.to_path_buf(),
        message: format!("{location_file} is not UTF-8. Nothing further was extracted."),
    })?;
    if xml.contains("<!DOCTYPE") || xml.contains("<!ENTITY") {
        return Err(Error::Extract {
            path: path.to_path_buf(),
            message: format!(
                "{location_file} contains a DOCTYPE or ENTITY declaration and was refused. Nothing was extracted."
            ),
        });
    }
    let mut reader = Reader::from_str(&xml);
    reader.config_mut().trim_text(false);
    let mut buf = Vec::new();
    let mut depth = 0usize;
    let mut in_text = false;
    let mut paragraph_no = 0usize;
    let mut current = String::new();
    let mut in_paragraph = false;
    loop {
        if reader.buffer_position() > MAX_ENTRY_BYTES {
            return Err(Error::Extract {
                path: path.to_path_buf(),
                message: format!("{location_file} is larger than the XML read limit."),
            });
        }
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => {
                depth += 1;
                if depth > MAX_XML_DEPTH {
                    return Err(Error::Extract {
                        path: path.to_path_buf(),
                        message: format!(
                            "{location_file} nests XML deeper than {MAX_XML_DEPTH}. Nothing further was extracted."
                        ),
                    });
                }
                let name = xml_name(&event);
                if is_local(&name, "p") {
                    in_paragraph = true;
                    paragraph_no += 1;
                    current.clear();
                } else if is_local(&name, "t") {
                    in_text = true;
                } else if is_local(&name, "tab") || is_local(&name, "br") || is_local(&name, "cr") {
                    current.push(' ');
                }
            }
            Ok(Event::Empty(event)) => {
                let name = xml_name_empty(&event);
                if is_local(&name, "tab") || is_local(&name, "br") || is_local(&name, "cr") {
                    current.push(' ');
                }
            }
            Ok(Event::Text(event)) => {
                if in_text {
                    let text = event.unescape().map_err(|err| Error::Extract {
                        path: path.to_path_buf(),
                        message: format!(
                            "{location_file} has XML text that could not be decoded ({err}). Custom entities are not expanded."
                        ),
                    })?;
                    current.push_str(&text);
                }
            }
            Ok(Event::End(event)) => {
                let name = xml_name_end(&event);
                if is_local(&name, "t") {
                    in_text = false;
                }
                if is_local(&name, "p") && in_paragraph {
                    let flat = flatten_block(&current);
                    if !flat.is_empty() {
                        locations.push(format!("{location_file} paragraph {paragraph_no}"));
                        blocks.push(flat);
                    }
                    current.clear();
                    in_paragraph = false;
                }
                depth = depth.saturating_sub(1);
            }
            Ok(Event::Eof) => break,
            Err(err) => {
                return Err(Error::Extract {
                    path: path.to_path_buf(),
                    message: format!("{location_file} could not be parsed as XML ({err})."),
                });
            }
            _ => {}
        }
        buf.clear();
    }
    Ok(())
}

fn xml_name(event: &quick_xml::events::BytesStart<'_>) -> String {
    String::from_utf8_lossy(event.name().as_ref()).into_owned()
}

fn xml_name_empty(event: &quick_xml::events::BytesStart<'_>) -> String {
    xml_name(event)
}

fn xml_name_end(event: &quick_xml::events::BytesEnd<'_>) -> String {
    String::from_utf8_lossy(event.name().as_ref()).into_owned()
}

fn is_local(name: &str, local: &str) -> bool {
    name == local
        || name
            .strip_prefix('{')
            .and_then(|rest| rest.split_once('}'))
            .map(|(_, local_name)| local_name)
            == Some(local)
        || name.rsplit(':').next() == Some(local)
}

fn flatten_block(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("\n")
}

fn split_blocks(text: &str) -> Vec<String> {
    let normalized = text.replace('\r', "\n");
    let mut blocks = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    for line in normalized.split('\n') {
        if line.trim().is_empty() {
            if !current.is_empty() {
                blocks.push(current.join("\n"));
                current.clear();
            }
        } else {
            current.push(line.trim());
        }
    }
    if !current.is_empty() {
        blocks.push(current.join("\n"));
    }
    blocks
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn docx_with(document_xml: &str, extra: &[(&str, &[u8])]) -> Vec<u8> {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut cursor);
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            writer.start_file("word/document.xml", options).unwrap();
            writer.write_all(document_xml.as_bytes()).unwrap();
            for (name, bytes) in extra {
                writer.start_file(*name, options).unwrap();
                writer.write_all(bytes).unwrap();
            }
            writer.finish().unwrap();
        }
        cursor.into_inner()
    }

    #[test]
    fn docx_paragraphs_keep_locations_and_skip_images() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body>
    <w:p><w:r><w:t>Synthetic fixture.</w:t></w:r></w:p>
    <w:p><w:r><w:t>The beaker cooled from 60 C.</w:t></w:r></w:p>
    <w:p><w:r><w:t>结果是稳定的。</w:t></w:r></w:p>
  </w:body>
</w:document>"#;
        let bytes = docx_with(xml, &[("word/media/pic.png", b"not a real png")]);
        let extracted = extract_docx(Path::new("lab.docx"), &bytes).unwrap();
        assert_eq!(extracted.locations.len(), 3);
        assert!(extracted.locations[1].contains("word/document.xml paragraph 2"));
        assert!(extracted.text.contains("结果是稳定的"));
        assert!(extracted
            .diagnostics
            .iter()
            .any(|note| note.contains("1 embedded image")));
        assert!(extracted
            .diagnostics
            .iter()
            .any(|note| note.contains("OCR is not included")));
    }

    #[test]
    fn hostile_docx_is_refused_with_a_limit() {
        let mut cursor = Cursor::new(Vec::new());
        {
            let mut writer = ZipWriter::new(&mut cursor);
            let options =
                SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
            for index in 0..200 {
                writer
                    .start_file(format!("word/extra{index}.xml"), options)
                    .unwrap();
                writer.write_all(b"<w:t>x</w:t>").unwrap();
            }
            writer.finish().unwrap();
        }
        let error = extract_docx(Path::new("many.docx"), &cursor.into_inner()).unwrap_err();
        let message = error.to_string();
        assert!(message.contains("128"), "{message}");
        assert!(message.contains("Nothing was extracted"), "{message}");

        let xml = r#"<?xml version="1.0"?>
<!DOCTYPE foo [<!ENTITY a "aaaa">]>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
  <w:body><w:p><w:r><w:t>&a;</w:t></w:r></w:p></w:body>
</w:document>"#;
        let bytes = docx_with(xml, &[]);
        let error = extract_docx(Path::new("entity.docx"), &bytes).unwrap_err();
        assert!(error.to_string().contains("DOCTYPE") || error.to_string().contains("ENTITY"));

        let bytes = docx_with("<w:document/>", &[("../secret.txt", b"nope")]);
        let extracted = extract_docx(Path::new("slip.docx"), &bytes).unwrap();
        assert!(extracted
            .diagnostics
            .iter()
            .any(|note| note.contains("unsafe path")));
        assert!(!extracted.text.contains("nope"));

        let huge = "A".repeat(MAX_ENTRY_BYTES as usize + 32);
        let xml = format!(
            "<?xml version=\"1.0\"?><w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:body><w:p><w:r><w:t>{huge}</w:t></w:r></w:p></w:body></w:document>"
        );
        let bytes = docx_with(&xml, &[]);
        let error = extract_docx(Path::new("huge.docx"), &bytes).unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("uncompressed") || message.contains("XML"),
            "{message}"
        );
    }

    fn ascii_pdf(text: &str) -> Vec<u8> {
        let escaped = text
            .replace('\\', "\\\\")
            .replace('(', "\\(")
            .replace(')', "\\)");
        let stream = format!("BT /F1 12 Tf 72 700 Td ({escaped}) Tj ET\n");
        let mut body = String::from("%PDF-1.4\n");
        let mut offsets = Vec::new();
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_string(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_string(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Contents 4 0 R /Resources << /Font << /F1 5 0 R >> >> >>".to_string(),
            format!("<< /Length {} >>\nstream\n{stream}endstream", stream.len()),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_string(),
        ];
        for (index, object) in objects.iter().enumerate() {
            offsets.push(body.len());
            body.push_str(&format!("{} 0 obj\n{object}\nendobj\n", index + 1));
        }
        let xref_at = body.len();
        body.push_str("xref\n0 6\n0000000000 65535 f \n");
        for offset in offsets {
            body.push_str(&format!("{offset:010} 00000 n \n"));
        }
        body.push_str(&format!(
            "trailer\n<< /Size 6 /Root 1 0 R >>\nstartxref\n{xref_at}\n%%EOF\n"
        ));
        body.into_bytes()
    }

    #[test]
    fn text_pdf_records_the_page_and_empty_pdf_says_so() {
        let extracted = extract_pdf(
            Path::new("lab.pdf"),
            &ascii_pdf("Synthetic cooling note for a text layer page that is long enough."),
        )
        .unwrap();
        assert!(extracted.text.contains("Synthetic cooling note"));
        assert!(extracted.locations[0].starts_with("page 1"));
        assert!(extracted
            .diagnostics
            .iter()
            .any(|note| note.contains("OCR is not included")));

        let empty = extract_pdf(Path::new("blank.pdf"), &ascii_pdf("")).unwrap();
        assert!(empty.text.is_empty() || empty.text.trim().is_empty());
        assert!(empty
            .diagnostics
            .iter()
            .any(|note| note.contains("no text layer") || note.contains("No extractable text")));
    }

    #[test]
    fn malformed_pdf_explains_itself() {
        let error = extract_pdf(Path::new("bad.pdf"), b"%PDF-1.4\nnot a pdf").unwrap_err();
        let message = error.to_string();
        assert!(message.contains("PDF"), "{message}");
        assert!(message.contains("OCR was not run"), "{message}");
    }
}
