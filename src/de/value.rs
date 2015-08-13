
use value::{Element, Content};
use error::{Error, ErrorCode};
use serde::de;
use std::{vec, mem};
use std::collections::btree_map;

pub struct Deserializer {
    value: Option<Element>,
}

impl Deserializer {
    /// Creates a new deserializer instance for deserializing the specified JSON value.
    pub fn new(value: Element) -> Deserializer {
        Deserializer {
            value: Some(value),
        }
    }
}
impl de::Deserializer for Deserializer {
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        use self::MapDeserializerState::*;
        debug!("value::Deserializer::visit {:?}\n", self.value);
        let el = match self.value.take() {
            Some(value) => value,
            None => { return Err(de::Error::end_of_stream()); }
        };

        match (el.attributes.is_empty(), el.members) {
            (true, Content::Text(s)) => visitor.visit_string(s),
            (true, Content::Nothing) => visitor.visit_unit(),
            (_, m) => visitor.visit_map( MapDeserializer {
                attributes: el.attributes
                              .into_iter()
                              .map(|(k, v)| (k, v.into_iter()))
                              .collect(),
                state: Inner,
                members: m,
            }),
        }
    }

    #[inline]
    fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        debug!("value::Deserializer::visit_option\n");
        if self.value.is_none() {
            return Err(de::Error::end_of_stream());
        };
        if self.value == Some(Element::new_empty()) {
            visitor.visit_none()
        } else {
            visitor.visit_some(self)
        }
    }

    #[inline]
    fn visit_enum<V>(&mut self, _name: &str, _variants: &'static [&'static str], mut visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        debug!("value::Deserializer::visit_enum\n");
        visitor.visit(VariantVisitor(self.value.take()))
    }

    #[inline]
    fn visit_map<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        use self::MapDeserializerState::*;
        debug!("value::Deserializer::visit_map {:?}\n", self.value);
        let el = match self.value.take() {
            Some(value) => value,
            None => { return Err(de::Error::end_of_stream()); }
        };
        visitor.visit_map( MapDeserializer {
            attributes: el.attributes
                          .into_iter()
                          .map(|(k, v)| (k, v.into_iter()))
                          .collect(),
            state: Inner,
            members: el.members,
        })
    }
}

struct VariantVisitor(Option<Element>);

impl de::VariantVisitor for VariantVisitor
{
    type Error = Error;

    fn visit_variant<V>(&mut self) -> Result<V, Self::Error>
        where V: de::Deserialize
    {
        debug!("VariantVisitor::visit_variant\n");
        if let Some(s) = self.0.as_mut().unwrap().attributes.remove("xsi:type") {
            de::Deserialize::deserialize(&mut StringDeserializer(s.into_iter().next()))
        } else {
            return Err(Error::SyntaxError(ErrorCode::Expected("attribute xsi:type"), 0, 0));
        }
    }

    /// `visit_unit` is called when deserializing a variant with no values.
    fn visit_unit(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }

    /// `visit_tuple` is called when deserializing a tuple-like variant.
    fn visit_tuple<V>(&mut self, _len: usize, _visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        unimplemented!()
    }

    /// `visit_struct` is called when deserializing a struct-like variant.
    fn visit_struct<V>(&mut self, _fields: &'static [&'static str], mut visitor: V) -> Result<V::Value, Self::Error>
        where V: de::Visitor
    {
        debug!("VariantVisitor::visit_map\n");
        let el = self.0.take().unwrap();
        visitor.visit_map(MapDeserializer {
            attributes: el.attributes
                          .into_iter()
                          .map(|(k, v)| (k, v.into_iter()))
                          .collect(),
            state: MapDeserializerState::Inner,
            members: el.members,
        })
    }
}

struct SeqDeserializer<I: Iterator<Item=Element> + ExactSizeIterator>(I);

impl<I> de::Deserializer for SeqDeserializer<I>
    where I: Iterator<Item=Element>,
    I: ExactSizeIterator,
{
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        debug!("seqdeserializer::visit\n");
        if let Some(el) = self.0.next() {
            debug!("el\n");
            de::Deserialize::deserialize(&mut Deserializer::new(el))
        } else {
            debug!("unit\n");
            visitor.visit_unit()
        }
    }

    #[inline]
    fn visit_enum<V>(&mut self, _name: &str, _variants: &'static [&'static str], mut visitor: V) -> Result<V::Value, Error>
        where V: de::EnumVisitor,
    {
        debug!("value::Deserializer::visit_enum\n");
        visitor.visit(VariantVisitor(self.0.next()))
    }

    #[inline]
    fn visit_seq<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        debug!("seqdeserializer::visit_seq\n");
        visitor.visit_seq(self)
    }
}

impl<I> de::SeqVisitor for SeqDeserializer<I>
    where I: Iterator<Item=Element>,
    I: ExactSizeIterator,
{
    type Error = Error;

    fn visit<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize
    {
        debug!("SeqDeserializer::visit\n");
        match self.0.next() {
            Some(value) => {
                debug!("value: {:?}\n", value);
                de::Deserialize::deserialize(&mut Deserializer::new(value))
                    .map(|v| Some(v))
            }
            None => Ok(None),
        }
    }

    fn end(&mut self) -> Result<(), Error> {
        debug!("SeqDeserializer::end\n");
        if self.0.len() == 0 {
            Ok(())
        } else {
            // FIXME: should be trailing whitespace error?
            Err(de::Error::end_of_stream())
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.len(), Some(self.0.len()))
    }
}

struct StringDeserializer(Option<String>);

impl de::Deserializer for StringDeserializer {
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        visitor.visit_string(self.0.take().unwrap())
    }
}

#[derive(PartialEq, Debug)]
enum MapDeserializerState {
    Inner,
    Done,
}

struct MapDeserializer {
    attributes: btree_map::BTreeMap<String, vec::IntoIter<String>>,
    members: Content,
    state: MapDeserializerState,
}

impl de::MapVisitor for MapDeserializer {
    type Error = Error;

    fn visit_key<T>(&mut self) -> Result<Option<T>, Error>
        where T: de::Deserialize
    {
        Ok(None)
    }

    fn visit_value<T>(&mut self) -> Result<T, Error>
        where T: de::Deserialize
    {
        unreachable!()
    }

    fn end(&mut self) -> Result<(), Error> {
        debug!("value::MapDeserializer::end\n");
        Ok(())
    }

    fn missing_field<V>(&mut self, field: &'static str) -> Result<V, Error>
        where V: de::Deserialize,
    {
        use self::MapDeserializerState::*;
        debug!("value::MapDeserializer::missing_field {:?} {}\n", self.state, field);

        // See if the type can deserialize from a unit.
        struct UnitDeserializer;

        impl de::Deserializer for UnitDeserializer {
            type Error = Error;

            fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
                where V: de::Visitor,
            {
                visitor.visit_unit()
            }

            fn visit_option<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
                where V: de::Visitor,
            {
                visitor.visit_none()
            }
        }

        match self.state {
            Inner if field == "$value" => {
                debug!("value\n");
                self.state = Done;
                match mem::replace(&mut self.members, Content::Nothing) {
                    Content::Text(s) =>
                        de::Deserialize::deserialize(&mut StringDeserializer(Some(s))),
                    Content::Nothing =>
                        de::Deserialize::deserialize(&mut UnitDeserializer),
                    Content::Members(_) => Err(Error::MissingFieldError("inner text")),
                }
            },
            Inner => if let Some(v) = self.attributes.remove(field) {
                debug!("attr\n");
                de::Deserialize::deserialize(&mut SeqDeserializer(v.map(|s| Element::new_text(s))))
            } else if let Content::Members(ref mut m) = self.members {
                if let Some(el) = m.remove(field) {
                    debug!("el: {:?}\n", el);
                    de::Deserialize::deserialize(&mut SeqDeserializer(el.into_iter()))
                } else {
                    de::Deserialize::deserialize(&mut UnitDeserializer)
                }
            } else {
                de::Deserialize::deserialize(&mut UnitDeserializer)
            },
            Done => de::Deserialize::deserialize(&mut UnitDeserializer),
        }
    }
}

impl de::Deserializer for MapDeserializer {
    type Error = Error;

    #[inline]
    fn visit<V>(&mut self, mut visitor: V) -> Result<V::Value, Error>
        where V: de::Visitor,
    {
        debug!("MapDeserializer!\n");
        visitor.visit_map(self)
    }
}
