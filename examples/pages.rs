extern crate printpdf;

use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

fn main() {
    // To prevent empty documents, you must specify at least one page with one layer
    // You can later on add more pages with the add_page() function
    // You also have to specify the title of the PDF and the document creator
    let mut doc = PdfDocument::new("printpdf page test");
    let mut page1 = PdfPage::new(Mm(210.0), Mm(297.0));
    let layer1 = PdfLayer::new("Layer 1");
    page1.add_layer(layer1);
    doc.add_page(page1);

    // You can add more pages and layers to the PDF.
    // Just make sure you don't lose the references, otherwise, you can't add things to the layer anymore
    let mut page2 = PdfPage::new(Mm(210.0), Mm(297.0));
    let layer2 = PdfLayer::new("Layer 2");
    page2.add_layer(layer2);
    doc.add_page(page2);

    // If this is successful, you should see a PDF with two blank A4 pages
    doc.save(&mut BufWriter::new(File::create("test_pages.pdf").unwrap()))
        .unwrap();
}
