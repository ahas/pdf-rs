extern crate printpdf;

use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::iter::FromIterator;

fn main() {
    let mut doc = PdfDocument::new("printpdf graphics test");
    let mut page1 = PdfPage::new(Mm(297.0), Mm(297.0));
    let mut layer1 = PdfLayer::new("Layer 1");

    // Quadratic shape. The "false" determines if the next (following)
    // point is a bezier handle (for curves)
    // If you want holes, simply reorder the winding of the points to be
    // counterclockwise instead of clockwise.
    let points1 = vec![
        (Point::new(Mm(100.0), Mm(100.0)), false),
        (Point::new(Mm(100.0), Mm(200.0)), false),
        (Point::new(Mm(300.0), Mm(200.0)), false),
        (Point::new(Mm(300.0), Mm(100.0)), false),
    ];

    // Is the shape stroked? Is the shape closed? Is the shape filled?
    let line1 = Line {
        points: points1,
        is_closed: true,
        has_fill: true,
        has_stroke: true,
        is_clipping_path: false,
    };

    // Triangle shape
    // Note: Line is invisible by default, the previous method of
    // constructing a line is recommended!
    let mut line2 = Line::from_iter(vec![
        (Point::new(Mm(150.0), Mm(150.0)), false),
        (Point::new(Mm(150.0), Mm(250.0)), false),
        (Point::new(Mm(350.0), Mm(250.0)), false),
    ]);

    line2.set_closed(false);
    line2.set_stroke(true);
    line2.set_fill(false);
    line2.set_as_clipping_path(false);

    let fill_color = Color::Cmyk(Cmyk::new(0.0, 0.23, 0.0, 0.0, None));
    let outline_color = Color::Rgb(Rgb::new(0.75, 1.0, 0.64, None));
    let mut dash_pattern = LineDashPattern::default();
    dash_pattern.dash_1 = Some(20);

    layer1.set_fill_color(fill_color);
    layer1.set_outline_color(outline_color);
    layer1.set_outline_thickness(10.0);

    // Draw first line
    layer1.add_shape(line1);
    let fill_color_2 = Color::Cmyk(Cmyk::new(0.0, 0.0, 0.0, 0.0, None));
    let outline_color_2 = Color::Greyscale(Greyscale::new(0.45, None));

    // More advanced graphical options
    let gs = ExtendedGraphicsStateBuilder::new()
        .with_blend_mode(BlendMode::Seperable(SeperableBlendMode::Multiply))
        .with_overprint_stroke(true)
        .build();
    let gs = doc.embed(&gs).unwrap();
    let gs = page1.register(&gs);

    layer1.set_graphics_state(&gs);
    layer1.set_line_dash_pattern(dash_pattern);
    layer1.set_line_cap_style(LineCapStyle::Round);
    layer1.set_line_join_style(LineJoinStyle::Round);
    layer1.set_fill_color(fill_color_2);
    layer1.set_outline_color(outline_color_2);
    layer1.set_outline_thickness(15.0);

    // draw second line
    layer1.add_shape(line2);

    page1.add_layer(layer1);
    doc.add_page(page1);

    // If this is successful, you should see a PDF two shapes, one rectangle
    // and a dotted line
    doc.save(&mut BufWriter::new(
        File::create("test_graphics.pdf").unwrap(),
    ))
    .unwrap();
}
