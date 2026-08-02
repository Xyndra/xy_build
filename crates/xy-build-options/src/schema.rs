use std::collections::HashMap;

use xy_build_parser::Value::{self};

pub trait Parseable {
    fn parse(value: &Value) -> Result<Self, String>
    where
        Self: Sized;

    fn options() -> Option<Vec<String>>;
}

pub struct StringValue(pub String);
impl Parseable for StringValue {
    fn parse(value: &Value) -> Result<Self, String> {
        match value {
            Value::UnknownIdent(s) => Ok(StringValue(s.clone())),
            _ => Err("this needs to be a quoted value".to_string()),
        }
    }

    fn options() -> Option<Vec<String>> {
        None
    }
}

pub struct Remainder(pub HashMap<String, Value>);
impl Parseable for Remainder {
    fn parse(_value: &Value) -> Result<Self, String> {
        return Err("remainder should not be parsed".to_string());
    }

    fn options() -> Option<Vec<String>> {
        None
    }
}

#[macro_export]
macro_rules! parseable_enum {
    ($enum_name: ident {$($variant: ident),*}) => {
        pub enum $enum_name {
            $($variant,)*
        }

        impl Parseable for $enum_name {
            fn parse(value: &xy_build_parser::Value) -> Result<Self, String> {
                match value {
                    xy_build_parser::Value::KnownIdent(s) => {
                        match s.as_str() {
                            $(stringify!($variant) => Ok($enum_name::$variant),)*
                            _ => Err(format!("unknown variant: {}", s)),
                        }
                    }
                    _ => Err("this needs to be an unquoted, known value".to_string()),
                }
            }

            fn options() -> Option<Vec<String>> {
                Some(vec![$(stringify!($variant).to_string()),*])
            }
        }
    };
}

// AI-generated (above was written by me, below was only edited by me):
// ---------------------------------------------------------------------------
// parseable_struct!
//
//   parseable_struct! {
//       Name {
//           field1: Type1,
//           field2: Vec<Type2>,
//           rest: Remainder,   // optional, at most one
//       }
//   }
//
// Normal fields become `Option<Type>` and are parsed from their key. A single
// `Remainder` field is not parsed from any key; instead all keys the parser
// did not recognize are collected into `Some(Remainder(map))`. Trailing commas
// and complex types (`Vec<String>` etc.) are supported. Two `Remainder` fields
// are rejected at compile time.
//
// The @split stage is a TT-muncher that classifies fields before emitting:
// the literal-token `Remainder` arms MUST come before the generic `$fty:ty`
// arms, because `:ty` would happily match the path `Remainder` and swallow it
// as a normal field.
// ---------------------------------------------------------------------------

#[macro_export]
macro_rules! parseable_struct {
    // ---- entry ----
    ($name:ident { $($fields:tt)* }) => {
        parseable_struct! {
            @split
            $name
            []
            []
            $($fields)*
        }
    };

    // ---- finished, no remainder ----
    (
        @split
        $name:ident
        [$($n:ident : $t:ty)*]
        []
    ) => {
        parseable_struct! {
            @emit
            $name
            [$($n : $t)*]
            []
        }
    };

    // ---- finished, with remainder ----
    (
        @split
        $name:ident
        [$($n:ident : $t:ty)*]
        [$r:ident]
    ) => {
        parseable_struct! {
            @emit
            $name
            [$($n : $t)*]
            [$r]
        }
    };

    // ---- first Remainder field (comma after) ----
    (
        @split
        $name:ident
        [$($n:ident : $t:ty)*]
        []
        $f:ident : Remainder,
        $($rest:tt)*
    ) => {
        parseable_struct! {
            @split
            $name
            [$($n : $t)*]
            [$f]
            $($rest)*
        }
    };

    // ---- first Remainder field (no trailing comma) ----
    (
        @split
        $name:ident
        [$($n:ident : $t:ty)*]
        []
        $f:ident : Remainder
    ) => {
        parseable_struct! {
            @split
            $name
            [$($n : $t)*]
            [$f]
        }
    };

    // ---- second Remainder field -> compile error ----
    (
        @split
        $name:ident
        [$($n:ident : $t:ty)*]
        [$e:ident]
        $f:ident : Remainder
        $(, $($rest:tt)*)?
    ) => {
        compile_error!("parseable_struct! may contain at most one Remainder field");
    };

    // ---- ordinary field (comma after) ----
    (
        @split
        $name:ident
        [$($n:ident : $t:ty)*]
        [$($r:tt)*]
        $f:ident : $fty:ty,
        $($rest:tt)*
    ) => {
        parseable_struct! {
            @split
            $name
            [$($n : $t)* $f : $fty]
            [$($r)*]
            $($rest)*
        }
    };

    // ---- ordinary field (no trailing comma) ----
    (
        @split
        $name:ident
        [$($n:ident : $t:ty)*]
        [$($r:tt)*]
        $f:ident : $fty:ty
    ) => {
        parseable_struct! {
            @split
            $name
            [$($n : $t)* $f : $fty]
            [$($r)*]
        }
    };

    // ---- emit, no remainder ----
    (
        @emit
        $name:ident
        [$($n:ident : $t:ty)*]
        []
    ) => {
        pub struct $name {
            $(pub $n: Option<$t>,)*
        }

        impl Parseable for $name {
            fn parse(value: &xy_build_parser::Value) -> Result<Self, String> {
                match value {
                    xy_build_parser::Value::Block(entries) => {
                        $(let mut $n = None;)*

                        for entry in entries {
                            match entry.key {
                                xy_build_parser::Key::KnownIdent(ref s) => match s.as_str() {
                                    $(
                                        stringify!($n) => {
                                            if $n.is_some() {
                                                return Err(format!("duplicate field: {}", stringify!($n)));
                                            }
                                            $n = Some(<$t as Parseable>::parse(&entry.value)?);
                                        }
                                    )*
                                    _ => return Err(format!("unknown field: {}", s)),
                                },
                                xy_build_parser::Key::UnknownIdent(ref s) => {
                                    return Err(format!("unknown field: {}", s));
                                }
                            }
                        }

                        Ok($name {
                            $($n,)*
                        })
                    }
                    _ => Err("this needs to be a block".to_string()),
                }
            }

            fn options() -> Option<Vec<String>> {
                Some(vec![$(stringify!($n).to_string()),*])
            }
        }
    };

    // ---- emit, with remainder ----
    (
        @emit
        $name:ident
        [$($n:ident : $t:ty)*]
        [$r:ident]
    ) => {
        pub struct $name {
            $(pub $n: Option<$t>,)*
            pub $r: Option<Remainder>,
        }

        impl Parseable for $name {
            fn parse(value: &xy_build_parser::Value) -> Result<Self, String> {
                match value {
                    xy_build_parser::Value::Block(entries) => {
                        let mut map = std::collections::HashMap::new();
                        $(let mut $n = None;)*

                        for entry in entries {
                            match entry.key {
                                xy_build_parser::Key::KnownIdent(ref s) => match s.as_str() {
                                    $(
                                        stringify!($n) => {
                                            if $n.is_some() {
                                                return Err(format!("duplicate field: {}", stringify!($n)));
                                            }
                                            $n = Some(<$t as Parseable>::parse(&entry.value)?);
                                        }
                                    )*
                                    _ => return Err(format!("unknown field: {}", s)),
                                },
                                xy_build_parser::Key::UnknownIdent(ref s) => {
                                    map.insert(s.clone(), entry.value.clone());
                                }
                            }
                        }

                        Ok($name {
                            $($n,)*
                            $r: Some(Remainder(map)),
                        })
                    }
                    _ => Err("this needs to be a block".to_string()),
                }
            }

            fn options() -> Option<Vec<String>> {
                Some(vec![$(stringify!($n).to_string()),*])
            }
        }
    };
}
