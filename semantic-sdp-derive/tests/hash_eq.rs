use semantic_sdp_derive::SdpEnum;

// Required by SdpEnum derive macro
#[derive(Debug)]
enum EnumParseError {
    #[allow(dead_code)]
    VariantNotFound,
}

#[non_exhaustive]
#[derive(Debug, Clone, SdpEnum)]
enum GroupSemantics {
    #[sdp("LS")]
    LipSynchronization,
    #[sdp("BUNDLE")]
    Bundle,
    #[sdp(default)]
    Unknown(String),
}

#[test]
fn parse_unknown_preserve_case() {
    use std::str::FromStr;

    let gs = GroupSemantics::from_str("FOO").unwrap();
    if let GroupSemantics::Unknown(s) = gs {
        assert_eq!(s, "FOO");
    } else {
        panic!("FOO was not parsed as GroupSemantics::Unknown");
    }
}

#[test]
fn enum_unknown_eq() {
    assert_eq!(GroupSemantics::Bundle, GroupSemantics::Bundle);
    assert_eq!(GroupSemantics::Bundle, GroupSemantics::Unknown("BUNDLE".to_owned()));
    assert_eq!(GroupSemantics::Unknown("BUNDLE".to_owned()), GroupSemantics::Bundle);
    assert_eq!(GroupSemantics::Bundle, GroupSemantics::Unknown("bundle".to_owned()));
    assert_eq!(GroupSemantics::Unknown("bundle".to_owned()), GroupSemantics::Bundle);
    assert_eq!(
        GroupSemantics::Unknown("BUNDLE".to_owned()),
        GroupSemantics::Unknown("BUNDLE".to_owned())
    );
    assert_eq!(
        GroupSemantics::Unknown("bundle".to_owned()),
        GroupSemantics::Unknown("BUNDLE".to_owned())
    );
    assert_eq!(
        GroupSemantics::Unknown("BUNDLE".to_owned()),
        GroupSemantics::Unknown("bundle".to_owned())
    );
}

#[test]
fn enum_hash_eq() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    fn h(v: GroupSemantics) -> u64 {
        let mut hasher = DefaultHasher::new();
        v.hash(&mut hasher);
        hasher.finish()
    }

    assert_eq!(h(GroupSemantics::Bundle), h(GroupSemantics::Bundle));
    assert_eq!(
        h(GroupSemantics::Bundle),
        h(GroupSemantics::Unknown("BUNDLE".to_owned()))
    );
    assert_eq!(
        h(GroupSemantics::Unknown("BUNDLE".to_owned())),
        h(GroupSemantics::Bundle)
    );
    assert_eq!(
        h(GroupSemantics::Bundle),
        h(GroupSemantics::Unknown("bundle".to_owned()))
    );
    assert_eq!(
        h(GroupSemantics::Unknown("bundle".to_owned())),
        h(GroupSemantics::Bundle)
    );
    assert_eq!(
        h(GroupSemantics::Unknown("BUNDLE".to_owned())),
        h(GroupSemantics::Unknown("BUNDLE".to_owned()))
    );
    assert_eq!(
        h(GroupSemantics::Unknown("bundle".to_owned())),
        h(GroupSemantics::Unknown("BUNDLE".to_owned()))
    );
    assert_eq!(
        h(GroupSemantics::Unknown("BUNDLE".to_owned())),
        h(GroupSemantics::Unknown("bundle".to_owned()))
    );
}
