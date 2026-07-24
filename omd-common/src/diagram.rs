use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::Write;

const PLANTUML_ALPHABET: &[u8; 64] =
    b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz-_";

/// Encode PlantUML source for `plantuml.com` SVG URLs (raw deflate + custom base64).
pub fn plantuml_encode(source: &str) -> String {
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(source.as_bytes())
        .expect("deflate write");
    let compressed = encoder.finish().expect("deflate finish");
    plantuml_base64_encode(&compressed)
}

/// Build a PlantUML SVG image URL for the given diagram source.
pub fn plantuml_svg_url(source: &str) -> String {
    format!(
        "https://www.plantuml.com/plantuml/svg/~1{}",
        plantuml_encode(source)
    )
}

fn plantuml_base64_encode(data: &[u8]) -> String {
    let mut out = String::with_capacity((data.len() * 4 / 3) + 4);
    let mut i = 0;
    while i < data.len() {
        let b1 = data[i];
        let b2 = if i + 1 < data.len() { data[i + 1] } else { 0 };
        let b3 = if i + 2 < data.len() { data[i + 2] } else { 0 };

        out.push(PLANTUML_ALPHABET[(b1 >> 2) as usize] as char);
        out.push(PLANTUML_ALPHABET[(((b1 & 0x3) << 4) | (b2 >> 4)) as usize] as char);
        out.push(PLANTUML_ALPHABET[(((b2 & 0xF) << 2) | (b3 >> 6)) as usize] as char);
        out.push(PLANTUML_ALPHABET[(b3 & 0x3F) as usize] as char);
        i += 3;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plantuml_encode_nonempty() {
        let encoded = plantuml_encode("@startuml\nAlice -> Bob\n@enduml");
        assert!(!encoded.is_empty());
        assert!(encoded.chars().all(|c| PLANTUML_ALPHABET.contains(&(c as u8))));
    }

    #[test]
    fn plantuml_url_starts_with_host() {
        let url = plantuml_svg_url("@startuml\n@enduml");
        assert!(url.starts_with("https://www.plantuml.com/plantuml/svg/~1"));
    }
}
