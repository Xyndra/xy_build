/// Trait for types that describe a configuration option.
pub trait OptionSchema {
    fn field_kind() -> FieldKind;
    fn schema() -> &'static ObjSchema {
        match Self::field_kind() {
            FieldKind::Object(s) => s,
            _ => panic!("not an object type"),
        }
    }
}

#[derive(Clone, Copy)]
pub enum FieldKind {
    Str,
    Enum(&'static [&'static str]),
    Object(&'static ObjSchema),
}

#[derive(Clone, Copy)]
pub struct Field {
    pub name: &'static str,
    pub doc: &'static str,
    pub kind: FieldKind,
    pub default: Option<&'static str>,
}

#[derive(Clone, Copy)]
pub struct ObjSchema {
    pub doc: &'static str,
    pub fields: &'static [Field],
    pub rest: Option<RestKind>,
}

#[derive(Clone, Copy)]
pub enum RestKind {
    Str,
    Object(&'static ObjSchema),
}

/// Lowercase the first character of a string.
pub fn decapitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) => c.to_lowercase().to_string() + chars.as_str(),
        None => String::new(),
    }
}

// ---------------------------------------------------------------------------
// Helpers for field-kind dispatch
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! make_field_kind {
    (String) => { $crate::schema::FieldKind::Str };
    (Enum $inner:ident) => {
        paste::paste! {
            $crate::schema::FieldKind::Enum(*[<$inner _VARIANTS>])
        }
    };
    (Object $inner:ident) => {
        paste::paste! {
            $crate::schema::FieldKind::Object(*[<$inner _SCHEMA>])
        }
    };
}

#[macro_export]
macro_rules! field_type {
    (String) => { String };
    (Enum $inner:ident) => { $inner };
    (Object $inner:ident) => { $inner };
}

/// Resolve the config-file name for an enum variant.
/// When given an explicit rename use it; otherwise decapitalize the ident.
#[macro_export]
macro_rules! variant_name {
    ($vname:ident) => {
        $crate::schema::decapitalize(stringify!($vname))
    };
    ($vname:ident $rename:literal) => {
        $rename.to_string()
    };
}

// ---------------------------------------------------------------------------
// define_options! macro
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! define_options {
    // ── Struct ──────────────────────────────────────────────────────────
    (
        $doc:literal
        $vis:vis struct $name:ident {
            $(
                $field_doc:literal
                $fvis:vis $fname:ident : $fkind:ident $(($finner:ident))?,
            )*
        }
    ) => {
        paste::paste! {
            #[doc = $doc]
            $vis struct $name {
                $(
                    #[doc = $field_doc]
                    $fvis $fname: Option<$crate::field_type!($fkind $($finner)?)>,
                )*
            }

            #[doc = concat!("Lazy schema for [`", stringify!($name), "`].")]
            #[allow(non_upper_case_globals)]
            $vis static [<$name _SCHEMA>]: std::sync::LazyLock<&'static $crate::schema::ObjSchema> =
                std::sync::LazyLock::new(|| {
                    Box::leak(Box::new($crate::schema::ObjSchema {
                        doc: $doc,
                        fields: Box::leak(
                            Box::new([
                                $(
                                    $crate::schema::Field {
                                        name: stringify!($fname),
                                        doc: $field_doc,
                                        kind: $crate::make_field_kind!($fkind $($finner)?),
                                        default: None,
                                    },
                                )*
                            ])
                        ),
                        rest: None,
                    }))
                });

            impl $crate::schema::OptionSchema for $name {
                fn field_kind() -> $crate::schema::FieldKind {
                    $crate::schema::FieldKind::Object(*[<$name _SCHEMA>])
                }
            }
        }
    };

    // ── Enum ────────────────────────────────────────────────────────────
    //   "doc" Variant,         — config name = decapitalize("Variant")
    //   "doc" Variant("name"), — config name = "name"
    (
        $doc:literal
        $vis:vis enum $name:ident {
            $(
                $variant_doc:literal
                $vname:ident $(($rename:literal))?,
            )*
        }
    ) => {
        paste::paste! {
            #[doc = $doc]
            $vis enum $name {
                $(
                    #[doc = $variant_doc]
                    $vname,
                )*
            }

            #[doc = concat!("Lazy variant names for [`", stringify!($name), "`].")]
            #[allow(non_upper_case_globals)]
            $vis static [<$name _VARIANTS>]: std::sync::LazyLock<&'static [&'static str]> =
                std::sync::LazyLock::new(|| {
                    {
                        let names: Vec<&'static str> = vec![
                            $({
                                let s: &'static str =
                                    $crate::variant_name!($vname $($rename)?).leak();
                                s
                            }),*
                        ];
                        Box::leak(names.into_boxed_slice())
                    }
                });

            impl $crate::schema::OptionSchema for $name {
                fn field_kind() -> $crate::schema::FieldKind {
                    $crate::schema::FieldKind::Enum(*[<$name _VARIANTS>])
                }
            }
        }
    };
}
