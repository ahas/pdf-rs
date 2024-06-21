//! Abstraction class for images.
//! Please use this class instead of adding `ImageXObjects` yourself

use image::{self, DynamicImage, ImageDecoder};
use std::borrow::Borrow;
use {Embeddable, ImageXObject, Registered};

/// Image - wrapper around an `ImageXObject` to allow for more control
/// within the library
#[derive(Debug)]
pub struct Image {
    /// The actual image
    pub image: ImageXObject,
}

impl From<ImageXObject> for Image {
    fn from(image: ImageXObject) -> Self {
        Self { image }
    }
}

#[cfg(feature = "image")]
impl Image {
    pub fn try_from<T: ImageDecoder>(image: T) -> Result<Self, image::ImageError> {
        let image = ImageXObject::try_from(image)?;
        Ok(Self { image })
    }

    pub fn from_dynamic_image(image: DynamicImage) -> Self {
        Self {
            image: ImageXObject::from_dynamic_image(image),
        }
    }
}

impl Embeddable for Image {
    const KEY: &'static str = "XObject";

    fn embed(&self, doc: &mut lopdf::Document) -> lopdf::Result<lopdf::ObjectId> {
        self.image.embed(doc)
    }
}

impl<I: Borrow<Image>> crate::types::RegisteredXObject for &Registered<I> {
    fn xobject_name(&self) -> Vec<u8> {
        let name = format!("R{}", self.name_index);
        name.as_bytes().to_vec()
    }
}
