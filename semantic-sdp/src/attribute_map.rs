use std::collections::HashMap;

use crate::attribute_types::{parse_attribute, NamedAttribute, ParsableAttribute};

// TODO: Might make sense to use smallvec here
// TODO: Box<dyn> is working out well, but it'd be good to look at the enum approach again
pub struct AttributeMap(HashMap<String, Vec<Box<dyn ParsableAttribute>>>);

impl std::fmt::Debug for AttributeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AttributeMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn insert<T: NamedAttribute>(&mut self, attribute: T) {
        self.0
            .entry(T::NAME.to_owned())
            .or_default()
            .push(Box::new(attribute));
    }

    // This skips check that the value is the right type, so we only allow it for internal
    // use where we expect the Box to have come from parse_attribute
    pub(crate) fn insert_boxed(&mut self, name: &str, value: Box<dyn ParsableAttribute>) {
        self.0.entry(name.to_owned()).or_default().push(value);
    }

    // We just use String as the result type to avoid exposing the nom trait soup publicly
    pub fn insert_unknown(&mut self, name: &str, value: Option<String>) -> Result<(), String> {
        // This is quite gross, but we appear to need it for safety. We could bypass it
        // for actually unknown attributes, but that'd need another list of them. We
        // won't be using this function, it's just for extensibility, so don't worry
        // about it for now.
        let value = match value {
            Some(value) => format!(":{}", value),
            None => "".to_owned(),
        };

        let (_, attribute) = parse_attribute(name, &value).map_err(|e| match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => {
                nom::error::convert_error(value.as_str(), e)
            }
            nom::Err::Incomplete(_) => unreachable!(),
        })?;

        self.0.entry(name.to_owned()).or_default().push(attribute);
        Ok(())
    }

    pub fn get<T: NamedAttribute>(&self) -> Option<&T> {
        self.0.get(T::NAME).and_then(|attributes| {
            let attribute = attributes.iter().next()?;
            let attribute = attribute.as_any();
            let attribute = attribute
                .downcast_ref()
                .expect("wrong type found in attribute bucket");
            Some(attribute)
        })
    }

    pub fn get_unknown(&self, name: &str) -> Option<Option<String>> {
        self.0.get(name).and_then(|attributes| {
            let attribute = attributes.iter().next()?;
            Some(attribute.to_string())
        })
    }

    pub fn get_vec<T: NamedAttribute>(&self) -> Vec<&T> {
        match self.0.get(T::NAME) {
            Some(attributes) => attributes
                .iter()
                .map(|attribute| {
                    let attribute = attribute.as_any();
                    let attribute = attribute
                        .downcast_ref()
                        .expect("wrong type found in attribute bucket");
                    attribute
                })
                .collect::<Vec<_>>(),
            None => Vec::new(),
        }
    }

    pub fn get_unknown_vec(&self, name: &str) -> Vec<Option<String>> {
        match self.0.get(name) {
            Some(attributes) => attributes
                .iter()
                .map(|attribute| attribute.to_string())
                .collect::<Vec<_>>(),
            None => Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::attribute_types::{IceLite, Mid, NamedAttribute};

    use super::AttributeMap;
    use crate::GroupSemantics;

    #[test]
    fn test_not_existing_known() {
        let map = AttributeMap::new();
        assert_eq!(map.get::<Mid>(), None);
    }

    #[test]
    fn test_not_existing_unknown() {
        let map = AttributeMap::new();
        assert_eq!(map.get_unknown("invalid"), None);
    }

    #[test]
    fn test_single_known() {
        let mut map = AttributeMap::new();
        map.insert(Mid("test".to_owned()));
        assert_eq!(map.get::<Mid>(), Some(&Mid("test".to_owned())));
    }

    #[test]
    fn test_single_unknown_property() {
        let mut map = AttributeMap::new();
        map.insert_unknown("invalid", None).unwrap();
        assert_eq!(map.get_unknown("invalid"), Some(None));
    }

    #[test]
    fn test_single_unknown_value() {
        let mut map = AttributeMap::new();
        map.insert_unknown("invalid", Some("value".to_owned()))
            .unwrap();
        assert_eq!(map.get_unknown("invalid"), Some(Some("value".to_owned())));
    }

    #[test]
    fn test_multiple_known() {
        let mut map = AttributeMap::new();
        map.insert(Mid("test".to_owned()));
        map.insert(Mid("test_two".to_owned()));
        assert_eq!(
            map.get_vec::<Mid>(),
            vec![&Mid("test".to_owned()), &Mid("test_two".to_owned())]
        );
    }

    #[test]
    fn test_multiple_unknown() {
        let mut map = AttributeMap::new();
        map.insert_unknown("invalid", Some("value".to_owned()))
            .unwrap();
        map.insert_unknown("invalid", Some("value_two".to_owned()))
            .unwrap();
        assert_eq!(
            map.get_unknown_vec("invalid"),
            vec![Some("value".to_owned()), Some("value_two".to_owned())]
        );
    }

    #[test]
    fn test_known_to_unknown_property() {
        let mut map = AttributeMap::new();
        map.insert(IceLite);
        assert_eq!(map.get_unknown(IceLite::NAME), Some(None));
    }

    #[test]
    fn test_known_to_unknown_value() {
        let mut map = AttributeMap::new();
        map.insert(Mid("test".to_owned()));
        assert_eq!(map.get_unknown(Mid::NAME), Some(Some("test".to_owned())));
    }

    #[test]
    fn test_unknown_to_known_property() {
        let mut map = AttributeMap::new();
        map.insert_unknown(IceLite::NAME, None).unwrap();
        assert_eq!(map.get::<IceLite>(), Some(&IceLite));
    }

    #[test]
    fn test_unknown_to_known_value() {
        let mut map = AttributeMap::new();
        map.insert_unknown(Mid::NAME, Some("test".to_owned()))
            .unwrap();
        assert_eq!(map.get::<Mid>(), Some(&Mid("test".to_owned())));
    }

    #[test]
    fn enum_unknown_eq() {
        assert_eq!(GroupSemantics::Bundle, GroupSemantics::Bundle);
        assert_eq!(
            GroupSemantics::Bundle,
            GroupSemantics::Unknown("BUNDLE".to_owned())
        );
        assert_eq!(
            GroupSemantics::Unknown("BUNDLE".to_owned()),
            GroupSemantics::Bundle
        );
    }
}
