use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Calamine error: {0}")]
    Calamine(String),
    
    #[error("XlsxWriter error: {0}")]
    XlsxWriter(String),
    
    #[error("XML error: {0}")]
    Xml(#[from] quick_xml::Error),
    
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    
    #[error("Serde error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
    
    #[error("Invalid configuration: {0}")]
    Config(String),
    
    #[error("Unknown error: {0}")]
    Unknown(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
