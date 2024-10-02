use lopdf;
use {OCGList, OCGRef, Pattern, PatternList, PatternRef};

/// Struct for storing the PDF Resources, to be used on a PDF page
#[derive(Default, Debug, Clone)]
pub struct PdfResources {
    /// Patterns used on this page. Do not yet, use, placeholder.
    pub patterns: PatternList,
    /// Layers / optional content ("Properties") in the resource dictionary
    pub layers: OCGList,
}

impl PdfResources {
    /// Creates a new PdfResources struct (resources for exactly one PDF page)
    pub fn new() -> Self {
        Self::default()
    }

    /// __STUB__: Adds a pattern to the resources, to be used like a color
    #[inline]
    pub fn add_pattern(&mut self, pattern: Pattern) -> PatternRef {
        self.patterns.add_pattern(pattern)
    }

    /// See `XObject::Into_with_document`.
    /// The resources also need access to the layers (the optional content groups), this should be a
    /// `Vec<lopdf::Object::Reference>` (to the actual OCG groups, which are added on the document level)
    #[cfg_attr(feature = "clippy", allow(needless_return))]
    pub fn into_with_layers(self, layers: Vec<lopdf::Object>) -> (lopdf::Dictionary, Vec<OCGRef>) {
        let mut dict = lopdf::Dictionary::new();

        let mut ocg_dict = self.layers;
        let mut ocg_references = Vec::<OCGRef>::new();

        let patterns_dict: lopdf::Dictionary = self.patterns.into();

        if !layers.is_empty() {
            for l in layers {
                ocg_references.push(ocg_dict.add_ocg(l));
            }

            let cur_ocg_dict_obj: lopdf::Dictionary = ocg_dict.into();

            if cur_ocg_dict_obj.len() > 0 {
                dict.set("Properties", lopdf::Object::Dictionary(cur_ocg_dict_obj));
            }
        }

        if patterns_dict.len() > 0 {
            dict.set("Pattern", lopdf::Object::Dictionary(patterns_dict));
        }

        return (dict, ocg_references);
    }
}
