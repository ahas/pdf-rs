extern crate printpdf;
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

fn main() {
    let mut doc = PdfDocument::new("PDF_Document_title");
    let mut page = PdfPage::new(Mm(500.0), Mm(300.0));
    let mut layer = PdfLayer::new("Layer 1");

    let text = "Lorem ipsum";
    let text2 = "dolor sit amet";

    let mut font_reader =
        std::io::Cursor::new(include_bytes!("../assets/fonts/RobotoMedium.ttf").as_ref());

    let font = doc.add_external_font(&mut font_reader).unwrap();

    // `use_text` is a wrapper around making a simple string
    layer.use_text(text, 48.0, Mm(10.0), Mm(200.0), &doc, &font);

    // text fill color = blue
    let blue = Rgb::new(13.0 / 256.0, 71.0 / 256.0, 161.0 / 256.0, None);
    let orange = Rgb::new(244.0 / 256.0, 67.0 / 256.0, 54.0 / 256.0, None);
    layer.set_fill_color(Color::Rgb(blue));
    layer.set_outline_color(Color::Rgb(orange));

    // For more complex layout of text, you can use functions
    // defined on the PdfLayerReference
    // Make sure to wrap your commands
    // in a `begin_text_section()` and `end_text_section()` wrapper
    layer.begin_text_section();
    {
        // setup the general fonts.
        // see the docs for these functions for details
        layer.set_font(&font, 33.0);
        layer.set_text_cursor(Mm(10.0), Mm(100.0));
        layer.set_line_height(33.0);
        layer.set_word_spacing(3000.0);
        layer.set_character_spacing(10.0);

        // write two lines (one line break)
        layer.write_text(text, &doc, &font);
        layer.add_line_break();
        layer.write_text(text2, &doc, &font);
        layer.add_line_break();

        layer.set_text_rendering_mode(TextRenderingMode::FillStroke);
        layer.set_character_spacing(0.0);
        layer.set_text_matrix(TextMatrix::Rotate(10.0));

        // write one line, but write text2 in superscript
        layer.write_text(text, &doc, &font);
        layer.set_line_offset(10.0);
        layer.set_text_rendering_mode(TextRenderingMode::Stroke);
        layer.set_font(&font, 18.0);
        layer.write_text(text2, &doc, &font);
    }
    layer.end_text_section();

    page.add_layer(layer);
    doc.add_page(page);

    doc.save(&mut BufWriter::new(File::create("test_fonts.pdf").unwrap()))
        .unwrap();
}
