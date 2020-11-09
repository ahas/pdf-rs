//! PDF page management

use lopdf;
use ImageXObject;

use {
    ExtendedGraphicsState, ExtendedGraphicsStateRef, Mm, Pattern, PatternRef, PdfLayer,
    PdfResources, Pt, XObject, XObjectRef,
};

/// PDF page
#[derive(Debug, Clone)]
pub struct PdfPage {
    /// page width in point
    pub width: Pt,
    /// page height in point
    pub height: Pt,
    /// Page layers
    pub layers: Vec<PdfLayer>,
    /// Resources used in this page
    pub(crate) resources: PdfResources,
}

impl PdfPage {
    /// Create a new page, notice that width / height are in millimeter.
    /// Page must contain at least one layer
    pub fn new(width: Mm, height: Mm) -> Self {
        Self {
            width: width.into(),
            height: height.into(),
            layers: Vec::new(),
            resources: PdfResources::new(),
        }
    }

    /// Iterates through the layers attached to this page and gathers all resources,
    /// which the layers need. Then returns a dictonary with all the resources
    /// (fonts, image XObjects, etc.)
    ///
    /// While originally I had planned to build a system where you can reference contents
    /// from all over the document, this turned out to be a problem, because each type had
    /// to be handled differently (PDF weirdness)
    ///
    /// `layers` should be a Vec with all layers (optional content groups) that were added
    /// to the document on a document level, it should contain the indices of the layers
    /// (they will be ignored, todo) and references to the actual OCG dictionaries
    #[inline]
    pub(crate) fn collect_resources_and_streams(
        self,
        doc: &mut lopdf::Document,
        layers: &[(usize, lopdf::Object)],
    ) -> (lopdf::Dictionary, Vec<lopdf::Stream>) {
        let cur_layers = layers.iter().map(|l| l.1.clone()).collect();
        let (resource_dictionary, ocg_refs) = self
            .resources
            .into_with_document_and_layers(doc, cur_layers);

        // set contents
        let mut layer_streams = Vec::<lopdf::Stream>::new();
        use lopdf::content::Operation;
        use lopdf::Object::*;

        for (idx, mut layer) in self.layers.into_iter().enumerate() {
            // push OCG and q to the beginning of the layer
            layer
                .operations
                .insert(0, Operation::new("q".into(), vec![]));
            layer.operations.insert(
                0,
                Operation::new(
                    "BDC".into(),
                    vec![Name("OC".into()), Name(ocg_refs[idx].name.clone().into())],
                ),
            );

            // push OCG END and Q to the end of the layer stream
            layer.operations.push(Operation::new("Q".into(), vec![]));
            layer.operations.push(Operation::new("EMC".into(), vec![]));

            // should end up looking like this:

            // /OC /MC0 BDC
            // q
            // <layer stream content>
            // Q
            // EMC

            let layer_stream = layer.into();
            layer_streams.push(layer_stream);
        }

        (resource_dictionary, layer_streams)
    }

    /// Change the graphics state. Before this operation is done, you should save
    /// the graphics state using the `save_graphics_state()` function. This will change the
    /// current graphics state until the end of the page or until the page is reset to the
    /// previous state.
    /// Returns the old graphics state, in case it was overwritten, as well as a reference
    /// to the currently active graphics state
    #[inline]
    pub fn add_graphics_state(
        &mut self,
        added_state: ExtendedGraphicsState,
    ) -> ExtendedGraphicsStateRef {
        self.resources.add_graphics_state(added_state)
    }

    /// __STUB__: Adds a pattern to the pages resources
    #[inline]
    pub fn add_pattern(&mut self, pattern: Pattern) -> PatternRef {
        self.resources.add_pattern(pattern)
    }

    /// __STUB__: Adds an XObject to the pages resources.
    /// __NOTE__: Watch out for scaling. Your XObject might be invisible or only 1pt x 1pt big
    #[inline]
    pub fn add_xobject(&mut self, xobj: XObject) -> XObjectRef {
        self.resources.add_xobject(xobj)
    }

    /// Add a layer on top of this page.
    #[inline]
    pub fn add_layer(&mut self, layer: PdfLayer) {
        // TODO: validate used resources.

        self.layers.push(layer);
    }

    /// Add an image object to the page
    /// To be called from the `image.add_to_layer()` class (see `use_xobject` documentation)
    pub(crate) fn add_image<T>(&mut self, image: T) -> XObjectRef
    where
        T: Into<ImageXObject>,
    {
        self.add_xobject(XObject::Image(image.into()))
    }
}
