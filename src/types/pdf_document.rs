//! A `PDFDocument` represents the whole content of the file

use std::io::BufWriter;
use std::io::Write;
use utils::random_character_string_32;

use crate::OffsetDateTime;
use lopdf;

use {
    BuiltinFont, DirectFontRef, Error, ExternalFont, Font, FontList, IccProfileList,
    IndirectFontRef, PdfConformance, PdfMetadata, PdfPage,
};

/// PDF document
#[derive(Debug, Clone)]
pub struct PdfDocument {
    /// Pages of the document
    pub(super) pages: Vec<PdfPage>,
    /// Fonts used in this document
    pub fonts: FontList,
    /// ICC profiles used in the document
    pub(super) icc_profiles: IccProfileList,
    /// Inner PDF document
    pub(super) inner_doc: lopdf::Document,
    /// Document ID. Must be changed if the document is loaded / parsed from a file
    pub document_id: String,
    /// Metadata for this document
    pub metadata: PdfMetadata,
}

impl PdfDocument {
    // Make a new empty document.
    pub fn new<S: Into<String>>(document_title: S) -> Self {
        Self {
            pages: Vec::new(),
            document_id: random_character_string_32(),
            fonts: FontList::new(),
            icc_profiles: IccProfileList::new(),
            inner_doc: lopdf::Document::with_version("1.3"),
            metadata: PdfMetadata::new(document_title, 1, false, PdfConformance::X3_2002_PDF_1_3),
        }
    }
}

impl PdfDocument {
    // ----- BUILDER FUNCTIONS

    /// Changes the title on both the document info dictionary as well as the metadata
    #[inline]
    pub fn set_title<S>(&mut self, new_title: S) -> ()
    where
        S: Into<String>,
    {
        self.metadata.document_title = new_title.into();
    }

    /// Set the trapping of the document
    #[inline]
    pub fn set_trapping(&mut self, trapping: bool) {
        self.metadata.trapping = trapping
    }

    /// Sets the document ID (for comparing two PDF documents for equality)
    #[inline]
    pub fn set_document_id(&mut self, id: String) {
        self.metadata.xmp_metadata.document_id = id;
    }

    /// Set the version of the document
    #[inline]
    pub fn set_document_version(&mut self, version: u32) {
        self.metadata.document_version = version;
    }

    /// Changes the conformance of this document. It is recommended to call
    /// `check_for_errors()` after changing it.
    #[inline]
    pub fn set_conformance(&mut self, conformance: PdfConformance) {
        self.metadata.conformance = conformance;
    }

    /// Sets the creation date on the document.
    ///
    /// Per default, the creation date is set to the current time.
    #[inline]
    pub fn set_creation_date(&mut self, creation_date: OffsetDateTime) {
        self.metadata.creation_date = creation_date;
    }

    /// Sets the modification date on the document. Intended to be used when
    /// reading documents that already have a modification date.
    #[inline]
    pub fn set_mod_date(&mut self, mod_date: OffsetDateTime) {
        self.metadata.modification_date = mod_date;
    }

    // ----- ADD FUNCTIONS

    /// Add a page to the document
    #[inline]
    pub fn add_page(&mut self, page: PdfPage) {
        self.pages.push(page);
    }

    /// Add a font from a font stream
    pub fn add_external_font<R>(
        &mut self,
        font_stream: R,
    ) -> ::std::result::Result<IndirectFontRef, Error>
    where
        R: ::std::io::Read,
    {
        let last_font_index = self.fonts.len();
        let external_font = ExternalFont::new(font_stream, last_font_index)?;
        let external_font_name = external_font.face_name.clone();
        let font = Font::ExternalFont(external_font);

        let font_ref;

        let possible_ref = {
            font_ref = IndirectFontRef::new(external_font_name);
            match self.fonts.get_font(&font_ref) {
                Some(f) => Some(f.clone()),
                None => None,
            }
        };

        if possible_ref.is_some() {
            Ok(font_ref)
        } else {
            let direct_ref = DirectFontRef {
                inner_obj: self.inner_doc.new_object_id(),
                data: font,
            };

            self.fonts.add_font(font_ref.clone(), direct_ref);
            Ok(font_ref)
        }
    }

    /// Add a built-in font to the document
    ///
    /// Built-in fonts can only be used to print characters that are supported by the
    /// [Windows-1252][] encoding.  All other characters will be ignored.
    ///
    /// [Windows-1252]: https://en.wikipedia.org/wiki/Windows-1252
    pub fn add_builtin_font(
        &mut self,
        builtin_font: BuiltinFont,
    ) -> ::std::result::Result<IndirectFontRef, Error> {
        let builtin_font_name: &'static str = builtin_font.clone().into();

        let font_ref;

        let possible_ref = {
            font_ref = IndirectFontRef::new(builtin_font_name);
            match self.fonts.get_font(&font_ref) {
                Some(f) => Some(f.clone()),
                None => None,
            }
        };

        if possible_ref.is_some() {
            Ok(font_ref)
        } else {
            let direct_ref = DirectFontRef {
                inner_obj: self.inner_doc.new_object_id(),
                data: Font::BuiltinFont(builtin_font),
            };

            self.fonts.add_font(font_ref.clone(), direct_ref);
            Ok(font_ref)
        }
    }

    // ----- GET FUNCTIONS

    /// Returns a direct reference (object ID) to the font from an
    /// indirect reference (postscript name)
    #[inline]
    pub fn get_font(&self, font: &IndirectFontRef) -> Option<DirectFontRef> {
        self.fonts.get_font(font)
    }

    /// Drops the PDFDocument, returning the inner `lopdf::Document`.
    /// Document may be only half-written, use only in extreme cases
    #[inline]
    pub unsafe fn get_inner(self) -> lopdf::Document {
        self.inner_doc
    }

    // --- MISC FUNCTIONS

    /// Checks for invalid settings in the document
    pub fn check_for_errors(&self) -> ::std::result::Result<(), Error> {
        // TODO
        #[cfg(feature = "logging")]
        {
            warn!("Checking PDFs for errors is currently not supported!");
        }

        Ok(())
    }

    /// Tries to match the document to the given conformance.
    /// Errors only on an unrecoverable error.
    pub fn repair_errors(&self, _conformance: PdfConformance) -> ::std::result::Result<(), Error> {
        // TODO
        #[cfg(feature = "logging")]
        {
            warn!("Reparing PDFs is currently not supported!");
        }

        Ok(())
    }

    /// Save PDF Document, writing the contents to the target
    pub fn save<W: Write>(self, target: &mut BufWriter<W>) -> ::std::result::Result<(), Error> {
        use lopdf::Object::*;
        use lopdf::StringFormat::Literal;
        use lopdf::{Dictionary as LoDictionary, Object as LoObject};
        use std::iter::FromIterator;

        let mut doc = self.inner_doc;

        // todo: remove unwrap, handle error
        let pages_id = doc.new_object_id();

        // extra pdf infos
        let (xmp_metadata, document_info, icc_profile) = self.metadata.into_obj();

        let xmp_metadata_id = xmp_metadata.map(|m| doc.add_object(m.clone()));
        let document_info_id = doc.add_object(document_info);

        // add catalog
        let icc_profile_descr = "Commercial and special offset print acccording to ISO \
                                 12647-2:2004 / Amd 1, paper type 1 or 2 (matte or gloss-coated \
                                 offset paper, 115 g/m2), screen ruling 60/cm";
        let icc_profile_str = "Coated FOGRA39 (ISO 12647-2:2004)";
        let icc_profile_short = "FOGRA39";

        let mut output_intents = LoDictionary::from_iter(vec![
            ("S", Name("GTS_PDFX".into())),
            ("OutputCondition", String(icc_profile_descr.into(), Literal)),
            ("Type", Name("OutputIntent".into())),
            (
                "OutputConditionIdentifier",
                String(icc_profile_short.into(), Literal),
            ),
            (
                "RegistryName",
                String("http://www.color.org".into(), Literal),
            ),
            ("Info", String(icc_profile_str.into(), Literal)),
        ]);

        let mut catalog = LoDictionary::from_iter(vec![
            ("Type", "Catalog".into()),
            ("PageLayout", "OneColumn".into()),
            ("PageMode", "Use0".into()),
            ("Pages", Reference(pages_id)),
        ]);

        if let Some(profile) = icc_profile {
            let icc_profile: lopdf::Stream = profile.into();
            let icc_profile_id = doc.add_object(Stream(icc_profile));
            output_intents.set("DestinationOutputProfile", Reference(icc_profile_id));
            catalog.set("OutputIntents", Array(vec![Dictionary(output_intents)]));
        }

        if let Some(metadata_id) = xmp_metadata_id {
            catalog.set("Metadata", Reference(metadata_id));
        }

        let mut pages = LoDictionary::from_iter(vec![
            ("Type", "Pages".into()),
            ("Count", Integer(self.pages.len() as i64)),
            /* Kids and Resources missing */
        ]);

        // add all pages with contents
        let mut page_ids = Vec::<LoObject>::new();

        // ----- OCG CONTENT

        // page index + page names to add the OCG to the /Catalog
        let page_layer_names: Vec<(usize, Vec<::std::string::String>)> = self
            .pages
            .iter()
            .map(|page| page.layers.iter().map(|layer| layer.name.clone()).collect())
            .enumerate()
            .collect();

        // add optional content groups (layers) to the /Catalog
        let usage_ocg_dict = LoDictionary::from_iter(vec![
            ("Type", Name("OCG".into())),
            (
                "CreatorInfo",
                Dictionary(LoDictionary::from_iter(vec![
                    ("Creator", String("Adobe Illustrator 14.0".into(), Literal)),
                    ("Subtype", Name("Artwork".into())),
                ])),
            ),
        ]);

        let usage_ocg_dict_ref = doc.add_object(Dictionary(usage_ocg_dict));

        let intent_arr = Array(vec![Name("View".into()), Name("Design".into())]);

        let intent_arr_ref = doc.add_object(intent_arr);

        // page index, layer index, reference to OCG dictionary
        let ocg_list: Vec<(usize, Vec<(usize, lopdf::Object)>)> = page_layer_names
            .into_iter()
            .map(|(page_idx, layer_names)| {
                (
                    page_idx,
                    layer_names
                        .into_iter()
                        .map(|layer_name| {
                            Reference(doc.add_object(Dictionary(LoDictionary::from_iter(vec![
                                ("Type", Name("OCG".into())),
                                ("Name", String(layer_name.into(), Literal)),
                                ("Intent", Reference(intent_arr_ref)),
                                ("Usage", Reference(usage_ocg_dict_ref)),
                            ]))))
                        })
                        .enumerate()
                        .collect(),
                )
            })
            .collect();

        let flattened_ocg_list: Vec<lopdf::Object> = ocg_list
            .iter()
            .flat_map(|&(_, ref layers)| layers.iter().map(|&(_, ref obj)| obj.clone()))
            .collect();

        catalog.set(
            "OCProperties",
            Dictionary(LoDictionary::from_iter(vec![
                ("OCGs", Array(flattened_ocg_list.clone())),
                // optional content configuration dictionary, page 376
                (
                    "D",
                    Dictionary(LoDictionary::from_iter(vec![
                        ("Order", Array(flattened_ocg_list.clone())),
                        // "radio button groups"
                        ("RBGroups", Array(vec![])),
                        // initially visible OCG
                        ("ON", Array(flattened_ocg_list)),
                    ])),
                ),
            ])),
        );

        // ----- END OCG CONTENT (on document level)

        // ----- PAGE CONTENT

        // add fonts (shared resources)
        let mut font_dict_id = None;

        // add all fonts / other resources shared in the whole document
        let fonts_dict: lopdf::Dictionary = self.fonts.into_with_document(&mut doc);

        if fonts_dict.len() > 0 {
            font_dict_id = Some(doc.add_object(Dictionary(fonts_dict)));
        }

        for (idx, page) in self.pages.into_iter().enumerate() {
            let mut p = LoDictionary::from_iter(vec![
                ("Type", "Page".into()),
                ("Rotate", Integer(0)),
                (
                    "MediaBox",
                    vec![0.into(), 0.into(), page.width.into(), page.height.into()].into(),
                ),
                (
                    "TrimBox",
                    vec![0.into(), 0.into(), page.width.into(), page.height.into()].into(),
                ),
                (
                    "CropBox",
                    vec![0.into(), 0.into(), page.width.into(), page.height.into()].into(),
                ),
                ("Parent", Reference(pages_id)),
            ]);

            // this will collect the resources needed for rendering this page
            let layers_temp = ocg_list.iter().find(|e| e.0 == idx).unwrap();
            let (mut resources_page, layer_streams) =
                page.collect_resources_and_streams(&mut doc, &layers_temp.1);

            if let Some(f) = font_dict_id {
                resources_page.set("Font", Reference(f));
            }

            if resources_page.len() > 0 {
                let resources_page_id = doc.add_object(Dictionary(resources_page));
                p.set("Resources", Reference(resources_page_id));
            }

            // merge all streams of the individual layers into one big stream
            let mut layer_streams_merged_vec = Vec::<u8>::new();
            for mut stream in layer_streams {
                layer_streams_merged_vec.append(&mut stream.content);
            }

            let merged_layer_stream =
                lopdf::Stream::new(lopdf::Dictionary::new(), layer_streams_merged_vec)
                    .with_compression(false);
            let page_content_id = doc.add_object(merged_layer_stream);

            p.set("Contents", Reference(page_content_id));
            page_ids.push(Reference(doc.add_object(p)))
        }

        pages.set::<_, LoObject>("Kids".to_string(), page_ids.into());

        // ----- END PAGE CONTENT

        doc.objects.insert(pages_id, Dictionary(pages));

        // save inner document
        let catalog_id = doc.add_object(catalog);
        let instance_id = random_character_string_32();

        doc.trailer.set("Root", Reference(catalog_id));
        doc.trailer.set("Info", Reference(document_info_id));
        doc.trailer.set(
            "ID",
            Array(vec![
                String(self.document_id.as_bytes().to_vec(), Literal),
                String(instance_id.as_bytes().to_vec(), Literal),
            ]),
        );

        // does nothing in debug mode, optimized in release mode
        Self::optimize(&mut doc);
        doc.save_to(target)?;

        Ok(())
    }

    #[cfg(any(debug_assertions, feature = "less-optimization"))]
    #[inline]
    fn optimize(_: &mut lopdf::Document) {}

    #[cfg(all(not(debug_assertions), not(feature = "less-optimization")))]
    #[inline]
    fn optimize(doc: &mut lopdf::Document) {
        doc.prune_objects();
        doc.delete_zero_length_streams();
        doc.compress();
    }
}
