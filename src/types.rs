use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Tag {
    #[serde(rename = "_id")]
    pub(crate) id: String,
    pub(crate) name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Meme {
    #[serde(rename = "_id")]
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) tags: Vec<String>,
    pub(crate) image: String,
}

#[derive(Debug)]
pub struct MemeOutput {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) tags: Vec<String>,
    pub(crate) image: String,
}
