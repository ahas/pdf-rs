#![allow(trivial_numeric_casts)]

//! Embedding fonts in 2D for Pdf
use lopdf;
use lopdf::StringFormat;
use lopdf::{Dictionary as LoDictionary, Stream as LoStream};
use std::collections::BTreeMap;
use std::iter::FromIterator;
use types::pdf_resources::Embeddable;
use Error;

use rusttype::Codepoint as Cp;
use rusttype::FontCollection;
use rusttype::GlyphId as Gid;

/// The font
#[derive(Debug, Clone, PartialEq)]
pub enum Font {
    /// Represents one of the 14 built-in fonts (Arial, Helvetica, etc.)
    BuiltinFont(BuiltinFont),
    /// Represents a font loaded from an external file
    ExternalFont(ExternalFont),
}

impl Embeddable for Font {
    const KEY: &'static str = "Font";

    fn embed(&self, doc: &mut lopdf::Document) -> lopdf::Result<lopdf::ObjectId> {
        let o = match self {
            Font::ExternalFont(font) => font.into_with_document(doc),
            Font::BuiltinFont(font) => (*font).into(),
        };
        Ok(doc.add_object(o))
    }
}

/// Standard built-in PDF fonts
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum BuiltinFont {
    TimesRoman,
    TimesBold,
    TimesItalic,
    TimesBoldItalic,
    Helvetica,
    HelveticaBold,
    HelveticaOblique,
    HelveticaBoldOblique,
    Courier,
    CourierOblique,
    CourierBold,
    CourierBoldOblique,
    Symbol,
    ZapfDingbats,
}

impl Into<Font> for BuiltinFont {
    fn into(self) -> Font {
        Font::BuiltinFont(self)
    }
}

impl Into<&'static str> for BuiltinFont {
    fn into(self) -> &'static str {
        use BuiltinFont::*;
        match self {
            TimesRoman => "Times-Roman",
            TimesBold => "Times-Bold",
            TimesItalic => "Times-Italic",
            TimesBoldItalic => "Times-BoldItalic",
            Helvetica => "Helvetica",
            HelveticaBold => "Helvetica-Bold",
            HelveticaOblique => "Helvetica-Oblique",
            HelveticaBoldOblique => "Helvetica-BoldOblique",
            Courier => "Courier",
            CourierOblique => "Courier-Oblique",
            CourierBold => "Courier-Bold",
            CourierBoldOblique => "Courier-BoldOblique",
            Symbol => "Symbol",
            ZapfDingbats => "ZapfDingbats",
        }
    }
}

impl Into<LoDictionary> for BuiltinFont {
    fn into(self) -> LoDictionary {
        use lopdf::Object;
        use lopdf::Object::*;

        let font_id: &'static str = self.into();

        // Begin setting required font attributes
        let font_vec: Vec<(::std::string::String, Object)> = vec![
            ("Type".into(), Name("Font".into())),
            ("Subtype".into(), Name("Type1".into())),
            ("BaseFont".into(), Name(font_id.into())),
            ("Encoding".into(), Name("WinAnsiEncoding".into())),
            // Missing DescendantFonts and ToUnicode
        ];

        LoDictionary::from_iter(font_vec)
    }
}

impl Embeddable for BuiltinFont {
    const KEY: &'static str = "Font";

    fn embed(&self, doc: &mut lopdf::Document) -> lopdf::Result<lopdf::ObjectId> {
        let o: LoDictionary = (*self).into();
        Ok(doc.add_object(o))
    }
}

#[derive(Debug, Clone)]
pub struct ExternalFont {
    /// Font data
    pub(crate) font_bytes: Vec<u8>,
    /// Font name, for adding as a resource on the document
    // pub(crate) face_name: String,
    /// Is the font written vertically? Default: false
    pub(crate) vertical_writing: bool,
}

/// The text rendering mode determines how a text is drawn
/// The default rendering mode is `Fill`. The color of the
/// fill / stroke is determine by the current pages outline /
/// fill color.
///
/// See PDF Reference 1.7 Page 402
#[derive(Debug, Copy, Clone)]
pub enum TextRenderingMode {
    Fill,
    Stroke,
    FillStroke,
    Invisible,
    FillClip,
    StrokeClip,
    FillStrokeClip,
    Clip,
}

impl Into<i64> for TextRenderingMode {
    fn into(self) -> i64 {
        use TextRenderingMode::*;
        match self {
            Fill => 0,
            Stroke => 1,
            FillStroke => 2,
            Invisible => 3,
            FillClip => 4,
            StrokeClip => 5,
            FillStrokeClip => 6,
            Clip => 7,
        }
    }
}

impl ExternalFont {
    /// Creates a new font. The `index` is used for naming / identifying the font
    pub fn new<R>(mut font_stream: R) -> Result<Self, Error>
    where
        R: ::std::io::Read,
    {
        // read font from stream and parse font metrics
        let mut buf = Vec::<u8>::new();
        font_stream.read_to_end(&mut buf)?;

        // verify
        {
            let collection = FontCollection::from_bytes(buf.clone())?;
            let _font = collection
                .clone()
                .into_font()
                .unwrap_or(collection.font_at(0)?);
        };

        Ok(Self {
            font_bytes: buf,
            vertical_writing: false,
        })
    }

    /// Takes the font and adds it to the document and consumes the font
    pub(crate) fn into_with_document(&self, doc: &mut lopdf::Document) -> LoDictionary {
        use lopdf::Object;
        use lopdf::Object::*;

        let face_name = format!("Fo{}", doc.objects.len());

        let collection = FontCollection::from_bytes(&self.font_bytes).unwrap();
        let font = collection
            .clone()
            .into_font()
            .unwrap_or_else(|_| collection.font_at(0).unwrap());

        // Extract basic font information
        let face_metrics = font.v_metrics_unscaled();

        let font_stream = LoStream::new(
            LoDictionary::from_iter(vec![("Length1", Integer(self.font_bytes.len() as i64))]),
            self.font_bytes.clone(),
        )
        .with_compression(false); /* important! font stream must not be compressed! */

        // Begin setting required font attributes
        let mut font_vec: Vec<(::std::string::String, Object)> = vec![
            ("Type".into(), Name("Font".into())),
            ("Subtype".into(), Name("Type0".into())),
            ("BaseFont".into(), Name(face_name.clone().into_bytes())),
            // Identity-H for horizontal writing, Identity-V for vertical writing
            ("Encoding".into(), Name("Identity-H".into())),
            // Missing DescendantFonts and ToUnicode
        ];

        let mut font_descriptor_vec: Vec<(::std::string::String, Object)> = vec![
            ("Type".into(), Name("FontDescriptor".into())),
            ("FontName".into(), Name(face_name.clone().into_bytes())),
            ("Ascent".into(), Integer(face_metrics.ascent as i64)),
            ("Descent".into(), Integer(face_metrics.descent as i64)),
            ("CapHeight".into(), Integer(face_metrics.ascent as i64)),
            ("ItalicAngle".into(), Integer(0)),
            ("Flags".into(), Integer(32)),
            ("StemV".into(), Integer(80)),
        ];

        // End setting required font arguments

        // Maximum height of a single character in the font
        let mut max_height = 0;
        // Total width of all characters
        let mut total_width = 0;
        // Widths (or heights, depends on self.vertical_writing)
        // of the individual characters, indexed by glyph id
        let mut widths = Vec::<(u32, u32)>::new();

        // Glyph IDs - (Unicode IDs - character width, character height)
        let mut cmap = BTreeMap::<u32, (u32, u32, u32)>::new();
        cmap.insert(0, (0, 1000, 1000));

        for unicode in 0x0000..0xffff {
            let glyph = font.glyph(Cp(unicode));

            if glyph.id().0 == 0 {
                continue;
            }

            let glyph_id = glyph.id().0;
            let glyph = font.glyph(Gid(glyph_id));

            if let Some(glyph_metrics) = glyph.standalone().get_data() {
                let w = glyph_metrics.unit_h_metrics.advance_width;

                // Note: extents can be None, but then the character may still have a
                // horizontal advance!
                let h = glyph_metrics
                    .extents
                    .and_then(|extents| {
                        Some(extents.max.y - extents.min.y - face_metrics.descent as i32)
                    })
                    .unwrap_or(1000);

                if h > max_height {
                    max_height = h;
                };

                total_width += w as u32;
                cmap.insert(glyph_id, (unicode as u32, w as u32, h as u32));
            }
        }

        // Maps the character index to a unicode value - add this to the "ToUnicode" dictionary!
        //
        // To explain this structure: Glyph IDs have to be in segments where the first byte of the
        // first and last element have to be the same. A range from 0x1000 - 0x10FF is valid
        // but a range from 0x1000 - 0x12FF is not (0x10 != 0x12)
        // Plus, the maximum number of Glyph-IDs in one range is 100
        //
        // Since the glyph IDs are sequential, all we really have to do is to enumerate the vector
        // and create buckets of 100 / rest to 256 if needed

        let mut cur_first_bit: u16 = 0_u16; // current first bit of the glyph id (0x10 or 0x12) for example

        let mut all_cmap_blocks = Vec::new();

        {
            let mut current_cmap_block = Vec::new();

            for (glyph_id, unicode_width_tuple) in &cmap {
                if (*glyph_id >> 8) as u16 != cur_first_bit || current_cmap_block.len() >= 100 {
                    // end the current (beginbfchar endbfchar) block
                    all_cmap_blocks.push(current_cmap_block.clone());
                    current_cmap_block = Vec::new();
                    cur_first_bit = (*glyph_id >> 8) as u16;
                }

                let (unicode, width, _) = *unicode_width_tuple;
                current_cmap_block.push((*glyph_id, unicode));
                widths.push((*glyph_id, width));
            }

            all_cmap_blocks.push(current_cmap_block);
        }

        let cid_to_unicode_map = generate_cid_to_unicode_map(face_name.clone(), all_cmap_blocks);

        let cid_to_unicode_map_stream =
            LoStream::new(LoDictionary::new(), cid_to_unicode_map.as_bytes().to_vec());
        let cid_to_unicode_map_stream_id = doc.add_object(cid_to_unicode_map_stream);

        // encode widths / heights so that they fit into what PDF expects
        // see page 439 in the PDF 1.7 reference
        // basically widths_list will contain objects like this:
        // 20 [21, 99, 34, 25]
        // which means that the character with the GID 20 has a width of 21 units
        // and the character with the GID 21 has a width of 99 units
        let mut widths_list = Vec::<Object>::new();
        let mut current_low_gid = 0;
        let mut current_high_gid = 0;
        let mut current_width_vec = Vec::<Object>::new();

        // scale the font width so that it sort-of fits into an 1000 unit square
        let percentage_font_scaling = 1000.0 / (font.units_per_em() as f64);

        for (gid, width) in widths {
            if gid == current_high_gid {
                current_width_vec.push(Integer((width as f64 * percentage_font_scaling) as i64));
                current_high_gid += 1;
            } else {
                widths_list.push(Integer(current_low_gid as i64));
                widths_list.push(Array(current_width_vec.drain(..).collect()));

                current_width_vec.push(Integer((width as f64 * percentage_font_scaling) as i64));
                current_low_gid = gid;
                current_high_gid = gid + 1;
            }
        }
        // push the last widths, because the loop is delayed by one iteration
        widths_list.push(Integer(current_low_gid as i64));
        widths_list.push(Array(current_width_vec.drain(..).collect()));

        let w = {
            if self.vertical_writing {
                ("W2", Array(widths_list))
            } else {
                ("W", Array(widths_list))
            }
        };

        // default width for characters
        let dw = {
            if self.vertical_writing {
                ("DW2", Integer(1000))
            } else {
                ("DW", Integer(1000))
            }
        };

        let mut desc_fonts = LoDictionary::from_iter(vec![
            ("Type", Name("Font".into())),
            ("Subtype", Name("CIDFontType2".into())),
            ("BaseFont", Name(face_name.clone().into())),
            (
                "CIDSystemInfo",
                Dictionary(LoDictionary::from_iter(vec![
                    ("Registry", String("Adobe".into(), StringFormat::Literal)),
                    ("Ordering", String("Identity".into(), StringFormat::Literal)),
                    ("Supplement", Integer(0)),
                ])),
            ),
            w,
            dw,
        ]);

        let font_bbox = vec![
            Integer(0),
            Integer(max_height as i64),
            Integer(total_width as i64),
            Integer(max_height as i64),
        ];
        font_descriptor_vec.push(("FontFile2".into(), Reference(doc.add_object(font_stream))));

        // although the following entry is technically not needed, Adobe Reader needs it
        font_descriptor_vec.push(("FontBBox".into(), Array(font_bbox)));

        let font_descriptor_vec_id = doc.add_object(LoDictionary::from_iter(font_descriptor_vec));

        desc_fonts.set("FontDescriptor", Reference(font_descriptor_vec_id));

        font_vec.push((
            "DescendantFonts".into(),
            Array(vec![Dictionary(desc_fonts)]),
        ));
        font_vec.push(("ToUnicode".into(), Reference(cid_to_unicode_map_stream_id)));

        LoDictionary::from_iter(font_vec)
    }
}

impl Into<Font> for ExternalFont {
    fn into(self) -> Font {
        Font::ExternalFont(self)
    }
}

impl Embeddable for ExternalFont {
    const KEY: &'static str = "Font";

    fn embed(&self, doc: &mut lopdf::Document) -> lopdf::Result<lopdf::ObjectId> {
        let o = self.into_with_document(doc);
        Ok(doc.add_object(o))
    }
}

type GlyphId = u32;
type UnicodeCodePoint = u32;
type CmapBlock = Vec<(GlyphId, UnicodeCodePoint)>;

/// Generates a CMAP (character map) from valid cmap blocks
fn generate_cid_to_unicode_map(face_name: String, all_cmap_blocks: Vec<CmapBlock>) -> String {
    let mut cid_to_unicode_map = format!(
        include_str!("../../../../templates/gid_to_unicode_beg.txt"),
        face_name
    );

    for cmap_block in all_cmap_blocks
        .into_iter()
        .filter(|block| !block.is_empty() || block.len() < 100)
    {
        cid_to_unicode_map.push_str(format!("{} beginbfchar\r\n", cmap_block.len()).as_str());
        for (glyph_id, unicode) in cmap_block {
            cid_to_unicode_map.push_str(format!("<{:04x}> <{:04x}>\n", glyph_id, unicode).as_str());
        }
        cid_to_unicode_map.push_str("endbfchar\r\n");
    }

    cid_to_unicode_map.push_str(include_str!("../../../../templates/gid_to_unicode_end.txt"));
    cid_to_unicode_map
}

impl PartialEq for ExternalFont {
    /// Two fonts are equal if their names are equal, the contents aren't checked
    fn eq(&self, other: &ExternalFont) -> bool {
        self.font_bytes == other.font_bytes
    }
}
