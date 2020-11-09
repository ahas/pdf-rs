//! Shared types regarding the structure of a PDF.

pub mod pdf_conformance;
pub mod pdf_document;
pub mod pdf_layer;
pub mod pdf_metadata;
pub mod pdf_page;
pub mod plugins;

pub use self::pdf_conformance::{CustomPdfConformance, PdfConformance};
pub use self::pdf_document::PdfDocument;
pub use self::pdf_layer::PdfLayer;
pub use self::pdf_metadata::PdfMetadata;
pub use self::pdf_page::PdfPage;
pub use self::plugins::*;
