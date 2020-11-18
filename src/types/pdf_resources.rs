use std::ops::Deref;
use std::rc::Rc;

pub trait Embeddable {
    const KEY: &'static str;
    fn embed(&self, doc: &mut lopdf::Document) -> lopdf::Result<lopdf::ObjectId>;
}

impl<T> Embeddable for &T
where
    T: Embeddable,
{
    const KEY: &'static str = T::KEY;
    fn embed(&self, doc: &mut lopdf::Document) -> lopdf::Result<lopdf::ObjectId> {
        (*self).embed(doc)
    }
}

impl<T> Embeddable for Rc<T>
where
    T: Embeddable,
{
    const KEY: &'static str = T::KEY;
    fn embed(&self, doc: &mut lopdf::Document) -> lopdf::Result<lopdf::ObjectId> {
        self.as_ref().embed(doc)
    }
}

pub trait RegisteredXObject {
    fn xobject_name(&self) -> Vec<u8>;
}

#[derive(Debug, Copy, Clone)]
pub struct Embedded<T> {
    pub(crate) object: T,
    pub(crate) object_id: lopdf::ObjectId,
}

#[derive(Debug, Copy, Clone)]
pub struct Registered<T> {
    embedded: Embedded<T>,
    pub(crate) name_index: u32,
}

impl<T> Deref for Embedded<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.object
    }
}

impl<T> Deref for Registered<T> {
    type Target = Embedded<T>;

    fn deref(&self) -> &Self::Target {
        &self.embedded
    }
}

pub(crate) fn embed<T: Embeddable>(
    doc: &mut lopdf::Document,
    resource: T,
) -> lopdf::Result<Embedded<T>> {
    let object_id = resource.embed(doc)?;

    Ok(Embedded {
        object_id,
        object: resource,
    })
}

pub(crate) fn register<T: Embeddable + Clone>(
    resources: &mut lopdf::Dictionary,
    resource: &Embedded<T>,
) -> Registered<T> {
    let object_id = resource.object_id;

    let name_index = match resources.get_mut(T::KEY.as_bytes()) {
        Ok(lopdf::Object::Dictionary(dict)) => {
            let name_index = dict.len();
            let name = format!("R{}", name_index);
            dict.set(name, object_id);
            name_index
        }
        Err(lopdf::Error::DictKey) => {
            let mut dict = lopdf::Dictionary::new();
            let name_index = dict.len();
            let name = format!("R{}", name_index);
            dict.set(name, object_id);
            resources.set(T::KEY, dict);
            name_index
        }
        _ => {
            // error
            panic!("expected a PDF Dictionary");
        }
    };

    Registered {
        embedded: Embedded::clone(resource),
        name_index: name_index as _,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;

    struct X;

    impl X {
        fn hi(&self) {}
    }

    impl Embeddable for X {
        const KEY: &'static str = "xx";
        fn embed(&self, doc: &mut lopdf::Document) -> lopdf::Result<lopdf::ObjectId> {
            Ok(doc.new_object_id())
        }
    }

    #[derive(Clone)]
    struct Y;

    impl Y {
        fn hi(&self) {}
    }

    impl Embeddable for Y {
        const KEY: &'static str = "xx";
        fn embed(&self, doc: &mut lopdf::Document) -> lopdf::Result<lopdf::ObjectId> {
            Ok(doc.new_object_id())
        }
    }

    #[test]
    fn test_with_ref() {
        let x = X;

        let mut docu = lopdf::Document::new();
        let x = embed(&mut docu, &x).unwrap();
        x.hi();

        let mut dict = lopdf::Dictionary::new();
        let _rx = register(&mut dict, &x);
        let rx = register(&mut dict, &x);
        dbg!(&dict);
        rx.hi();

        let mut dict = lopdf::Dictionary::new();
        let rx = register(&mut dict, &x);
        dbg!(&dict);
        rx.hi();
    }

    #[test]
    fn test_with_owned() {
        let x = Y;

        let mut docu = lopdf::Document::new();
        let x = embed(&mut docu, x).unwrap();
        x.hi();

        let mut dict = lopdf::Dictionary::new();
        let _rx = register(&mut dict, &x);
        let rx = register(&mut dict, &x);
        dbg!(&dict);
        rx.hi();

        let mut dict = lopdf::Dictionary::new();
        let rx = register(&mut dict, &x);
        dbg!(&dict);
        rx.hi();
    }

    #[test]
    fn test_with_rc() {
        let x = Rc::new(X);

        let mut docu = lopdf::Document::new();
        let x = embed(&mut docu, x).unwrap();
        x.hi();

        let mut dict = lopdf::Dictionary::new();
        let _rx = register(&mut dict, &x);
        let rx = register(&mut dict, &x);
        dbg!(&dict);
        rx.hi();

        let mut dict = lopdf::Dictionary::new();
        let rx = register(&mut dict, &x);
        dbg!(&dict);
        rx.hi();
    }
}
