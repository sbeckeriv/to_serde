use wasm_bindgen::prelude::*;
mod to_serde;
pub mod serde_xml {
    use super::*;

    #[wasm_bindgen]
    pub fn parse_xml(xml: &str) -> String {
        to_serde::serde_xml::parse_xml_file(xml)
    }
}
