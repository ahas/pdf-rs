//! Example on how to use a built-in font in a PDF

extern crate printpdf;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

fn main() {
    let mut doc = PdfDocument::new("PDF_Document_title");
    doc.set_conformance(PdfConformance::Custom(CustomPdfConformance {
        requires_icc_profile: false,
        requires_xmp_metadata: false,
        ..Default::default()
    }));

    let mut page = PdfPage::new(Mm(500.0), Mm(300.0));

    {
        let mut layer = PdfLayer::new("Layer 1");

        let text = "Lorem ipsum";

        let font = doc.add_builtin_font(BuiltinFont::TimesBoldItalic).unwrap();
        layer.use_text(text, 48.0, Mm(10.0), Mm(200.0), &doc, &font);

        page.add_layer(layer);
    }

    doc.add_page(page);

    doc.save(&mut BufWriter::new(
        File::create("test_builtin_fonts.pdf").unwrap(),
    ))
    .unwrap();
}
