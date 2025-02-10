use std::path::PathBuf;

#[derive(Clone)]
pub enum Texture {
    Raw(Vec<u8>),
    Path(PathBuf),
}
