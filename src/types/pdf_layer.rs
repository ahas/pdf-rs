//! PDF layer management. Layers can contain referenced or real content.

use lopdf;

use glob_defines::OP_PATH_STATE_SET_LINE_WIDTH;
use lopdf::content::Operation;
use {
    Color, CurTransMat, ExtendedGraphicsStateRef, Font, IndirectFontRef, Line, LineCapStyle,
    LineDashPattern, LineJoinStyle, Mm, PdfColor, PdfDocument, Pt, TextMatrix, TextRenderingMode,
    XObjectRef,
};

/// One layer of PDF data
#[derive(Debug, Clone)]
pub struct PdfLayer {
    /// Name of the layer. Must be present for the optional content group
    pub(crate) name: String,
    /// Stream objects in this layer. Usually, one layer == one stream
    pub(super) operations: Vec<Operation>,
}

impl PdfLayer {
    /// Create a new layer, with a name and what index the layer has in the page
    #[inline]
    pub fn new<S>(name: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            name: name.into(),
            operations: Vec::new(),
        }
    }
}

impl Into<lopdf::Stream> for PdfLayer {
    fn into(self) -> lopdf::Stream {
        use lopdf::{Dictionary, Stream};
        let stream_content = lopdf::content::Content {
            operations: self.operations,
        };

        // page contents may not be compressed (todo: is this valid for XObjects?)
        Stream::new(Dictionary::new(), stream_content.encode().unwrap()).with_compression(false)
    }
}

impl PdfLayer {
    /// Add a shape to the layer. Use `closed` to indicate whether the line is a closed line
    /// Use has_fill to determine if the line should be filled.
    pub fn add_shape(&mut self, line: Line) {
        let line_ops = line.into_stream_op();
        for op in line_ops {
            self.internal_add_operation(op);
        }
    }

    /// Begins a new text section
    /// You have to make sure to call `end_text_section` afterwards
    #[inline]
    pub fn begin_text_section(&mut self) -> () {
        self.internal_add_operation(Operation::new("BT", vec![]));
    }

    /// Ends a new text section
    /// Only valid if `begin_text_section` has been called
    #[inline]
    pub fn end_text_section(&mut self) -> () {
        self.internal_add_operation(Operation::new("ET", vec![]));
    }

    /// Set the current fill color for the layer
    #[inline]
    pub fn set_fill_color(&mut self, fill_color: Color) -> () {
        self.internal_add_operation(PdfColor::FillColor(fill_color));
    }

    /// Set the current font, only valid in a `begin_text_section` to
    /// `end_text_section` block
    #[inline]
    pub fn set_font(&mut self, font: &IndirectFontRef, font_size: f64) -> () {
        self.internal_add_operation(Operation::new(
            "Tf",
            vec![font.name.clone().into(), (font_size).into()],
        ));
    }

    /// Set the current line / outline color for the layer
    #[inline]
    pub fn set_outline_color(&mut self, color: Color) {
        self.internal_add_operation(PdfColor::OutlineColor(color));
    }

    /// Instantiate layers, forms and postscript items on the page
    /// __WARNING__: Object must be added to the same page, since the XObjectRef is just a
    /// String, essentially, it can't be checked that this is the case. The caller is
    /// responsible for ensuring this. However, you can use the `Image` struct
    /// and use `image.add_to(layer)`, which will essentially do the same thing, but ensures
    /// that the image is referenced correctly
    ///
    /// Function is limited to this library to ensure that outside code cannot call it
    pub(crate) fn use_xobject(
        &mut self,
        xobj: &XObjectRef,
        translate_x: Option<Mm>,
        translate_y: Option<Mm>,
        rotate_cw: Option<f64>,
        scale_x: Option<f64>,
        scale_y: Option<f64>,
    ) {
        // save graphics state
        self.save_graphics_state();

        // apply ctm if any
        let (mut t_x, mut t_y) = (Mm(0.0), Mm(0.0));
        let (mut s_x, mut s_y) = (0.0, 0.0);

        if let Some(tr_x) = translate_x {
            t_x = tr_x;
        }
        if let Some(tr_y) = translate_y {
            t_y = tr_y;
        }
        if let Some(sc_x) = scale_x {
            s_x = sc_x;
        }
        if let Some(sc_y) = scale_y {
            s_y = sc_y;
        }

        // translate, rotate, scale - order does not matter

        if t_x != Mm(0.0) || t_y != Mm(0.0) {
            let translate_ctm = CurTransMat::Translate(t_x, t_y);
            self.internal_add_operation(translate_ctm);
        }

        if let Some(rot) = rotate_cw {
            let rotate_ctm = CurTransMat::Rotate(rot);
            self.internal_add_operation(rotate_ctm);
        }

        if s_x != 0.0 || s_y != 0.0 {
            let scale_ctm = CurTransMat::Scale(s_x, s_y);
            self.internal_add_operation(scale_ctm);
        }

        // invoke object
        self.internal_invoke_xobject(xobj.name.clone());

        // restore graphics state
        self.restore_graphics_state();
    }

    /// Change the graphics state of the current layer
    pub fn set_graphics_state(&mut self, graphics_state: ExtendedGraphicsStateRef) {
        self.internal_add_operation(Operation::new(
            "gs",
            vec![lopdf::Object::Name(
                graphics_state.gs_name.as_bytes().to_vec(),
            )],
        ));
    }

    /// Set the current line thickness, in points
    ///
    /// __NOTE__: 0.0 is a special value, it does not make the line disappear, but rather
    /// makes it appear 1px wide across all devices
    #[inline]
    pub fn set_outline_thickness(&mut self, outline_thickness: f64) {
        use lopdf::Object::*;
        self.internal_add_operation(Operation::new(
            OP_PATH_STATE_SET_LINE_WIDTH,
            vec![Real(outline_thickness)],
        ));
    }

    /// Set the current line join style for outlines
    #[inline]
    pub fn set_line_join_style(&mut self, line_join: LineJoinStyle) {
        self.internal_add_operation(line_join);
    }

    /// Set the current line join style for outlines
    #[inline]
    pub fn set_line_cap_style(&mut self, line_cap: LineCapStyle) {
        self.internal_add_operation(line_cap);
    }

    /// Set the current line join style for outlines
    #[inline]
    pub fn set_line_dash_pattern(&mut self, dash_pattern: LineDashPattern) {
        self.internal_add_operation(dash_pattern);
    }

    /// Sets (adds to) the current transformation matrix
    /// Use `save_graphics_state()` and `restore_graphics_state()`
    /// to "scope" the transformation matrix to a specific function
    #[inline]
    pub fn set_ctm(&mut self, ctm: CurTransMat) {
        self.internal_add_operation(ctm);
    }

    /// Sets (replaces) the current text matrix
    /// This does not have to be scoped, since the matrix is replaced
    /// instead of concatenated to the current matrix. However,
    /// you should only call this function with in a block scoped by
    /// `begin_text_section()` and `end_text_section()`
    #[inline]
    pub fn set_text_matrix(&mut self, tm: TextMatrix) {
        self.internal_add_operation(tm);
    }

    /// Sets the position where the text should appear
    #[inline]
    pub fn set_text_cursor(&mut self, x: Mm, y: Mm) {
        let x_in_pt: Pt = x.into();
        let y_in_pt: Pt = y.into();
        self.internal_add_operation(Operation::new("Td", vec![x_in_pt.into(), y_in_pt.into()]));
    }

    /// If called inside a text block scoped by `begin_text_section` and
    /// `end_text_section`, moves the cursor to a new line. PDF does not have
    /// any concept of "alignment" except left-aligned text
    /// __Note:__ Use `set_line_height` earlier to set the line height first
    #[inline]
    pub fn add_line_break(&mut self) {
        self.internal_add_operation(Operation::new("T*", Vec::new()));
    }

    /// Sets the text line height inside a text block
    /// (must be called within `begin_text_block` and `end_text_block`)
    #[inline]
    pub fn set_line_height(&mut self, height: f64) {
        self.internal_add_operation(Operation::new("TL", vec![lopdf::Object::Real(height)]));
    }

    /// Sets the character spacing inside a text block
    /// Values are given in points. A value of 3 (pt) will increase
    /// the spacing inside a word by 3pt.
    #[inline]
    pub fn set_character_spacing(&mut self, spacing: f64) {
        self.internal_add_operation(Operation::new("Tc", vec![lopdf::Object::Real(spacing)]));
    }

    /// Sets the word spacing inside a text block.
    /// Same as `set_character_spacing`, just for words.
    /// __Note:__ This currently does not work for external
    /// fonts. External fonts are encoded with Unicode, and
    /// PDF does not recognize unicode fonts. It only
    /// recognizes builtin fonts done with PDFDoc encoding.
    /// However, the function itself is valid and _will work_
    /// with builtin fonts.
    #[inline]
    pub fn set_word_spacing(&mut self, spacing: f64) {
        self.internal_add_operation(Operation::new("Tw", vec![lopdf::Object::Real(spacing)]));
    }

    /// Sets the horizontal scaling (like a "condensed" font)
    /// Default value is 100 (regular scaling). Setting it to
    /// 50 will reduce the width of the written text by half,
    /// but stretch the text
    #[inline]
    pub fn set_text_scaling(&mut self, scaling: f64) {
        self.internal_add_operation(Operation::new("Tz", vec![lopdf::Object::Real(scaling)]));
    }

    /// Offsets the current text positon (used for superscript
    /// and subscript). To reset the superscript / subscript, call this
    /// function with 0 as the offset. For superscript, use a positive
    /// number, for subscript, use a negative number. This does not
    /// change the size of the font
    #[inline]
    pub fn set_line_offset(&mut self, offset: f64) {
        self.internal_add_operation(Operation::new("Ts", vec![lopdf::Object::Real(offset)]));
    }

    #[inline]
    pub fn set_text_rendering_mode(&mut self, mode: TextRenderingMode) {
        self.internal_add_operation(Operation::new(
            "Tr",
            vec![lopdf::Object::Integer(mode.into())],
        ));
    }

    /// Add text to the file at the current position by specifying font codepoints for an
    /// ExternalFont
    pub fn write_codepoints<I>(&mut self, codepoints: I)
    where
        I: IntoIterator<Item = u16>,
    {
        use lopdf::Object::*;
        use lopdf::StringFormat::Hexadecimal;

        let bytes = codepoints
            .into_iter()
            .flat_map(|x| {
                let [b0, b1] = x.to_be_bytes();
                std::iter::once(b0).chain(std::iter::once(b1))
            })
            .collect::<Vec<u8>>();

        self.internal_add_operation(Operation::new("Tj", vec![String(bytes, Hexadecimal)]));
    }

    /// Add text to the file at the current position by specifying
    /// font codepoints with additional kerning offset
    pub fn write_positioned_codepoints<I>(&mut self, codepoints: I)
    where
        I: IntoIterator<Item = (i64, u16)>,
    {
        use lopdf::Object::*;
        use lopdf::StringFormat::Hexadecimal;

        let mut list = Vec::new();

        for (pos, codepoint) in codepoints {
            if pos != 0 {
                list.push(Integer(pos));
            }
            let bytes = codepoint.to_be_bytes().to_vec();
            list.push(String(bytes, Hexadecimal));
        }

        self.internal_add_operation(Operation::new("TJ", vec![Array(list)]));
    }

    /// Add text to the file at the current position
    ///
    /// If the given font is a built-in font and the given text contains characters that are not
    /// supported by the [Windows-1252][] encoding, these characters will be ignored.
    ///
    /// [Windows-1252]: https://en.wikipedia.org/wiki/Windows-1252
    #[inline]
    pub fn write_text<S>(&mut self, text: S, doc: &PdfDocument, font: &IndirectFontRef) -> ()
    where
        S: Into<String>,
    {
        // NOTE: The unwrap() calls in this function are safe, since
        // we've already checked the font for validity when it was added to the document

        use lopdf::Object::*;
        use lopdf::StringFormat::Hexadecimal;

        let text = text.into();

        // we need to transform the characters into glyph ids and then add them to the layer

        // glyph IDs that make up this string

        // kerning for each glyph id. If no kerning is present, will be 0
        // must be the same length as list_gid
        // let mut kerning_data = Vec::<freetype::Vector>::new();

        let bytes: Vec<u8> = {
            use rusttype::Codepoint as Cp;
            use rusttype::FontCollection;

            if let Font::ExternalFont(face_direct_ref) = doc.fonts.get_font(font).unwrap().data {
                let mut list_gid = Vec::<u16>::new();
                let collection = FontCollection::from_bytes(&*face_direct_ref.font_bytes).unwrap();
                let font = collection
                    .clone()
                    .into_font()
                    .unwrap_or(collection.font_at(0).unwrap());

                // convert into list of glyph ids - unicode magic
                let char_iter = text.chars();

                for ch in char_iter {
                    // note: font.glyph will panic if the character is \0
                    // since that can't happen in Rust, I think we're safe here
                    let glyph = font.glyph(Cp(ch as u32));
                    list_gid.push(glyph.id().0 as u16);

                    // todo - kerning !!
                    // font.pair_kerning(scale, id, base_glyph.id());
                }

                list_gid
                    .iter()
                    .flat_map(|x| vec![(x >> 8) as u8, (x & 255) as u8])
                    .collect::<Vec<u8>>()
            } else {
                // For built-in fonts, we selected the WinAnsiEncoding, see the Into<LoDictionary>
                // implementation for BuiltinFont.
                lopdf::Document::encode_text(Some("WinAnsiEncoding"), &text)
            }
        };

        self.internal_add_operation(Operation::new("Tj", vec![String(bytes, Hexadecimal)]));
    }

    /// Saves the current graphic state
    #[inline]
    pub fn save_graphics_state(&mut self) {
        self.internal_add_operation(Operation::new("q", Vec::new()));
    }

    /// Restores the previous graphic state
    #[inline]
    pub fn restore_graphics_state(&mut self) {
        self.internal_add_operation(Operation::new("Q", Vec::new()));
    }

    /// Add text to the file, x and y are measure in millimeter from the bottom left corner
    ///
    /// If the given font is a built-in font and the given text contains characters that are not
    /// supported by the [Windows-1252][] encoding, these characters will be ignored.
    ///
    /// [Windows-1252]: https://en.wikipedia.org/wiki/Windows-1252
    #[inline]
    pub fn use_text<S>(
        &mut self,
        text: S,
        font_size: f64,
        x: Mm,
        y: Mm,
        doc: &PdfDocument,
        font: &IndirectFontRef,
    ) -> ()
    where
        S: Into<String>,
    {
        self.begin_text_section();
        self.set_font(font, font_size);
        self.set_text_cursor(x, y);
        self.write_text(text, doc, font);
        self.end_text_section();
    }

    // internal function to invoke an xobject
    fn internal_invoke_xobject(&mut self, name: String) {
        self.internal_add_operation(lopdf::content::Operation::new(
            "Do",
            vec![lopdf::Object::Name(name.as_bytes().to_vec())],
        ));
    }

    #[inline(always)]
    fn internal_add_operation<T>(&mut self, op: T)
    where
        T: Into<Operation>,
    {
        self.operations.push(op.into());
    }
}
