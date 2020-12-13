use ordered_multimap::ListOrderedMultimap;

use crate::attributes::{parse_attribute, NamedAttribute, ParsableAttribute};

// TODO: Might make sense to use smallvec here
// TODO: Box<dyn> is working out well, but it'd be good to look at the enum approach again
pub struct AttributeMap(ListOrderedMultimap<String, Box<dyn ParsableAttribute>>);

impl std::fmt::Debug for AttributeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for AttributeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for (name, attribute) in self {
            write!(f, "a={}", name)?;
            if let Some(value) = attribute.to_string() {
                write!(f, ":{}", value)?;
            }
            write!(f, "\r\n")?;
        }

        Ok(())
    }
}

impl Default for AttributeMap {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> IntoIterator for &'a AttributeMap {
    type Item = (&'a String, &'a Box<dyn ParsableAttribute>);
    type IntoIter = ordered_multimap::list_ordered_multimap::Iter<'a, String, Box<dyn ParsableAttribute>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl AttributeMap {
    pub fn new() -> Self {
        Self(ListOrderedMultimap::new())
    }

    pub fn append<T: NamedAttribute>(&mut self, attribute: T) {
        self.0.append(T::NAME.to_owned(), Box::new(attribute));
    }

    // This skips checking that the value is the right type, so we only allow it for internal
    // use where we expect the Box to have come from parse_attribute
    pub(crate) fn append_boxed(&mut self, name: String, value: Box<dyn ParsableAttribute>) {
        self.0.append(name, value);
    }

    // We just use String as the result type to avoid exposing the nom trait soup publicly
    pub fn append_unknown(&mut self, name: &str, value: Option<String>) -> Result<(), String> {
        let name = name.to_ascii_lowercase();

        // This is quite gross, but we appear to need it for safety. We could bypass it
        // for actually unknown attributes, but that'd need another list of them. We
        // won't be using this function, it's just for extensibility, so don't worry
        // about it for now.
        let value = match value {
            Some(value) => format!(":{}", value),
            None => "".to_owned(),
        };

        let (_, attribute) = parse_attribute(&name, &value).map_err(|e| match e {
            nom::Err::Error(e) | nom::Err::Failure(e) => nom::error::convert_error(value.as_str(), e),
            nom::Err::Incomplete(_) => unreachable!(),
        })?;

        self.append_boxed(name, attribute);
        Ok(())
    }

    pub fn get<T: NamedAttribute>(&self) -> Option<&T> {
        self.0.get(T::NAME).map(|attribute| {
            let attribute = attribute.as_any();
            let attribute = attribute.downcast_ref();
            attribute.expect("wrong type found in attribute bucket")
        })
    }

    pub fn get_unknown(&self, name: &str) -> Option<Option<String>> {
        self.0.get(name).map(|attribute| attribute.to_string())
    }

    pub fn get_vec<T: NamedAttribute>(&self) -> Vec<&T> {
        self.0
            .get_all(T::NAME)
            .map(|attribute| {
                let attribute = attribute.as_any();
                let attribute = attribute.downcast_ref().expect("wrong type found in attribute bucket");
                attribute
            })
            .collect()
    }

    pub fn get_unknown_vec(&self, name: &str) -> Vec<Option<String>> {
        self.0.get_all(name).map(|attribute| attribute.to_string()).collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::attributes::{IceLite, Mid, NamedAttribute};
    use crate::AttributeMap;

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
        map.append(Mid("test".into()));
        assert_eq!(map.get::<Mid>(), Some(&Mid("test".into())));
    }

    #[test]
    fn test_single_unknown_property() {
        let mut map = AttributeMap::new();
        map.append_unknown("invalid", None).unwrap();
        assert_eq!(map.get_unknown("invalid"), Some(None));
    }

    #[test]
    fn test_single_unknown_value() {
        let mut map = AttributeMap::new();
        map.append_unknown("invalid", Some("value".to_owned())).unwrap();
        assert_eq!(map.get_unknown("invalid"), Some(Some("value".to_owned())));
    }

    #[test]
    fn test_multiple_known() {
        let mut map = AttributeMap::new();
        map.append(Mid("test".into()));
        map.append(Mid("test_two".into()));
        assert_eq!(map.get_vec::<Mid>(), vec![&Mid("test".into()), &Mid("test_two".into())]);
    }

    #[test]
    fn test_multiple_unknown() {
        let mut map = AttributeMap::new();
        map.append_unknown("invalid", Some("value".to_owned())).unwrap();
        map.append_unknown("invalid", Some("value_two".to_owned())).unwrap();
        assert_eq!(
            map.get_unknown_vec("invalid"),
            vec![Some("value".to_owned()), Some("value_two".to_owned())]
        );
    }

    #[test]
    fn test_known_to_unknown_property() {
        let mut map = AttributeMap::new();
        map.append(IceLite);
        assert_eq!(map.get_unknown(IceLite::NAME), Some(None));
    }

    #[test]
    fn test_known_to_unknown_value() {
        let mut map = AttributeMap::new();
        map.append(Mid("test".into()));
        assert_eq!(map.get_unknown(Mid::NAME), Some(Some("test".to_owned())));
    }

    #[test]
    fn test_unknown_to_known_property() {
        let mut map = AttributeMap::new();
        map.append_unknown(IceLite::NAME, None).unwrap();
        assert_eq!(map.get::<IceLite>(), Some(&IceLite));
    }

    #[test]
    fn test_unknown_to_known_value() {
        let mut map = AttributeMap::new();
        map.append_unknown(Mid::NAME, Some("test".to_owned())).unwrap();
        assert_eq!(map.get::<Mid>(), Some(&Mid("test".into())));
    }
}
