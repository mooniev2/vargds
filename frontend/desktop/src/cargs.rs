use std::path::PathBuf;

#[derive(argh::FromArgs)]
/// vargds
pub struct CArgs {
    #[argh(option)]
    /// rom path
    pub rom: Option<PathBuf>,
}

pub fn from_env() -> CArgs {
    argh::from_env()
}
