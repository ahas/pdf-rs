extern crate printpdf;

use image::bmp::BmpDecoder;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::io::Cursor;

fn main() {
    let mut doc = PdfDocument::new("printpdf graphics test");
    let mut page = PdfPage::new(Mm(210.0), Mm(297.0));
    let mut layer = PdfLayer::new("Layer 1");

    // currently, the only reliable file formats are bmp/jpeg/png
    // this is an issue of the image library, not a fault of printpdf

    let image_bytes = include_bytes!("../assets/img/BMP_test.bmp");
    let mut reader = Cursor::new(image_bytes.as_ref());

    let decoder = BmpDecoder::new(&mut reader).unwrap();
    let image = Image::try_from(decoder).unwrap();
    let image = doc.embed(&image).unwrap();
    let image = page.register(&image);

    // layer,
    layer.use_image(&image, None, None, None, None, None, None);

    page.add_layer(layer);
    doc.add_page(page);

    doc.save(&mut BufWriter::new(File::create("test_image.pdf").unwrap()))
        .unwrap();
}
