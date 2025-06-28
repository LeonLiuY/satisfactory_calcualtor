use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SatisfactoryAsset {
    #[serde(rename = "Classes")]
    pub classes: Vec<super::satisfactory_adapter::SatisfactoryClass>,
}
