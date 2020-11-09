//! Example to demonstrate how to remove the default ICC profile
//! Look at the file size (compared to the other tests!)

extern crate printpdf;

use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

fn main() {
	// This code creates the most minimal PDF file with 1.2 KB
	// Currently, fonts need to use an embedded font, so if you need to write something, the file size
	// will still be bloated (because of the embedded font)
	// Also, OCG content is still enabled, even if you disable it here.
	let mut doc = PdfDocument::new("printpdf no_icc test");
	let mut page = PdfPage::new(Mm(297.0), Mm(210.0));
	let layer = PdfLayer::new("Layer 1");

	doc.set_conformance(PdfConformance::Custom(CustomPdfConformance {
		requires_icc_profile: false,
		requires_xmp_metadata: false,
		..Default::default()
	}));

	page.add_layer(layer);
	doc.add_page(page);

	doc.save(&mut BufWriter::new(
		File::create("test_no_icc.pdf").unwrap(),
	))
	.unwrap();
}
