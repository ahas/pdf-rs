extern crate printpdf;

use printpdf::*;
use std::fs::File;
use std::io::BufWriter;

fn main() {
    use printpdf::utils::{calculate_points_for_circle, calculate_points_for_rect};

    let mut doc = PdfDocument::new("printpdf circle test");
    let mut page1 = PdfPage::new(Mm(210.0), Mm(297.0));
    let mut layer1 = PdfLayer::new("Layer 1");

    let radius = Pt(40.0);
    let offset_x = Pt(10.0);
    let offset_y = Pt(50.0);

    let line = Line {
        points: calculate_points_for_circle(radius, offset_x, offset_y),
        is_closed: true,
        has_fill: true,
        has_stroke: true,
        is_clipping_path: false,
    };

    layer1.add_shape(line);

    let scale_x_rect = Pt(40.0);
    let scale_y_rect = Pt(10.0);
    let offset_x_rect = Pt(20.0);
    let offset_y_rect = Pt(5.0);

    let line = Line {
        points: calculate_points_for_rect(scale_x_rect, scale_y_rect, offset_x_rect, offset_y_rect),
        is_closed: true,
        has_fill: true,
        has_stroke: true,
        is_clipping_path: false,
    };

    layer1.add_shape(line);
    page1.add_layer(layer1);
    doc.add_page(page1);

    doc.save(&mut BufWriter::new(
        File::create("test_circle.pdf").unwrap(),
    ))
    .unwrap();
}
