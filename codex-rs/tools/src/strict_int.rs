//! Strict integer coercion for model-authored tool arguments.
//!
//! This module is Ratchet A of the tool-args ratchet campaign
//! (`docs/tool-args-ratchet-spec.md`, §3). Models routinely send
//! integer-valued tool arguments as JSON strings (for example
//! `{"timeout_ms": "120000"}`); plain `serde` rejects those with an
//! un-teachable type error. The `strict_*` helpers below replace the
//! default integer deserializers on the campaign's verified target fields
//! (§3.1) and accept exactly:
//!
//! - a JSON number, with the same accept/reject classification serde applies
//!   today for the field type — range-checked per type; out-of-range numbers,
//!   floats, booleans, `null`, arrays, and objects are rejected — or
//! - a JSON string containing only a base-10 integer literal: digits, with at
//!   most one leading `-` (on signed types). `"0123"` parses as 123; `"+12"`
//!   is rejected explicitly (JSON number syntax has no `+`), and `"1.5"`,
//!   `" 12"`, `"0x10"`, `""`, `"null"`, and overflowing digit runs are
//!   rejected with the §3.2 "expected an integer" error.
//!
//! The `_opt` variants additionally map JSON `null` to `None`; every other
//! value delegates to the plain visitor wrapped in `Some`. Fields wire up per
//! §3.2, coexisting with `serde(default)`:
//!
//! ```rust,ignore
//! #[serde(default, deserialize_with = "strict_u64")]
//! yield_time_ms: u64,
//!
//! #[serde(default, deserialize_with = "strict_u64_opt")]
//! timeout_ms: Option<u64>,
//! ```

use core::fmt;
use core::str::FromStr;

use serde::Deserializer;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;
use serde::de::{self};

/// Per-type accept/reject semantics shared by the plain and `_opt` visitors.
///
/// Implementations must mirror serde's default visitor for the concrete
/// integer type on the number path (identical range acceptance) and accept
/// strings only as base-10 integer literals (no leading `+`).
trait StrictInt: Sized {
    /// The Rust type name used in error messages, matching serde's
    /// `expecting` for the corresponding integer type.
    const TYPE_NAME: &'static str;

    /// Accept a non-negative integer token per the target type's range rules.
    fn from_u64(value: u64) -> Result<Self, String>;

    /// Accept a signed integer token per the target type's range rules.
    fn from_i64(value: i64) -> Result<Self, String>;

    /// Accept a string iff it is a base-10 integer literal (no leading `+`).
    fn from_literal(value: &str) -> Result<Self, String>;
}

/// Exact §3.2 error text for strings that are not integer literals.
const INTEGER_LITERAL_ERROR: &str =
    "expected an integer (JSON number, or a string containing only an integer literal)";

fn invalid_value_int(value: &dyn fmt::Display, expected: &str) -> String {
    format!("invalid value: integer `{value}`, expected {expected}")
}

fn invalid_type_message(unexpected: &str, expected: &str) -> String {
    format!("invalid type: {unexpected}, expected {expected}")
}

/// Render a float the way serde's `Unexpected::Float` does (finite values
/// always show a decimal point).
fn float_token(value: f64) -> String {
    let rendered = format!("{value}");
    if value.is_finite() && !rendered.contains('.') {
        format!("{rendered}.0")
    } else {
        rendered
    }
}

fn parse_integer_literal<T: FromStr>(value: &str) -> Result<T, String> {
    // JSON number syntax has no leading `+`; reject it explicitly because
    // `FromStr` would accept it (spec §3.2, round-1 Major B4).
    if value.starts_with('+') {
        return Err(INTEGER_LITERAL_ERROR.to_string());
    }
    value
        .parse::<T>()
        .map_err(|_| INTEGER_LITERAL_ERROR.to_string())
}

impl StrictInt for u64 {
    const TYPE_NAME: &'static str = "u64";

    fn from_u64(value: u64) -> Result<Self, String> {
        Ok(value)
    }

    fn from_i64(value: i64) -> Result<Self, String> {
        if value >= 0 {
            Ok(value as u64)
        } else {
            Err(invalid_value_int(&value, Self::TYPE_NAME))
        }
    }

    fn from_literal(value: &str) -> Result<Self, String> {
        parse_integer_literal(value)
    }
}

impl StrictInt for usize {
    const TYPE_NAME: &'static str = "usize";

    fn from_u64(value: u64) -> Result<Self, String> {
        usize::try_from(value).map_err(|_| invalid_value_int(&value, Self::TYPE_NAME))
    }

    fn from_i64(value: i64) -> Result<Self, String> {
        if value >= 0 {
            usize::try_from(value as u64).map_err(|_| invalid_value_int(&value, Self::TYPE_NAME))
        } else {
            Err(invalid_value_int(&value, Self::TYPE_NAME))
        }
    }

    fn from_literal(value: &str) -> Result<Self, String> {
        parse_integer_literal(value)
    }
}

impl StrictInt for i32 {
    const TYPE_NAME: &'static str = "i32";

    fn from_u64(value: u64) -> Result<Self, String> {
        i32::try_from(value).map_err(|_| invalid_value_int(&value, Self::TYPE_NAME))
    }

    fn from_i64(value: i64) -> Result<Self, String> {
        i32::try_from(value).map_err(|_| invalid_value_int(&value, Self::TYPE_NAME))
    }

    fn from_literal(value: &str) -> Result<Self, String> {
        parse_integer_literal(value)
    }
}

impl StrictInt for i64 {
    const TYPE_NAME: &'static str = "i64";

    fn from_u64(value: u64) -> Result<Self, String> {
        if value <= i64::MAX as u64 {
            Ok(value as i64)
        } else {
            Err(invalid_value_int(&value, Self::TYPE_NAME))
        }
    }

    fn from_i64(value: i64) -> Result<Self, String> {
        Ok(value)
    }

    fn from_literal(value: &str) -> Result<Self, String> {
        parse_integer_literal(value)
    }
}

/// Visitor for the plain `strict_*` helpers: numbers keep serde's
/// accept/reject classification for the target type; strings must be
/// integer literals.
#[derive(Default)]
struct StrictIntVisitor<T: StrictInt> {
    _marker: std::marker::PhantomData<T>,
}

impl<'de, T: StrictInt + 'de> Visitor<'de> for StrictIntVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(T::TYPE_NAME)
    }

    fn visit_u64<E>(self, value: u64) -> Result<T, E>
    where
        E: de::Error,
    {
        T::from_u64(value).map_err(E::custom)
    }

    fn visit_i64<E>(self, value: i64) -> Result<T, E>
    where
        E: de::Error,
    {
        T::from_i64(value).map_err(E::custom)
    }

    fn visit_f64<E>(self, value: f64) -> Result<T, E>
    where
        E: de::Error,
    {
        Err(E::custom(invalid_type_message(
            &format!("floating point `{}`", float_token(value)),
            T::TYPE_NAME,
        )))
    }

    fn visit_bool<E>(self, value: bool) -> Result<T, E>
    where
        E: de::Error,
    {
        Err(E::custom(invalid_type_message(
            &format!("boolean `{value}`"),
            T::TYPE_NAME,
        )))
    }

    fn visit_str<E>(self, value: &str) -> Result<T, E>
    where
        E: de::Error,
    {
        T::from_literal(value).map_err(E::custom)
    }

    fn visit_bytes<E>(self, _value: &[u8]) -> Result<T, E>
    where
        E: de::Error,
    {
        Err(E::custom(invalid_type_message("byte array", T::TYPE_NAME)))
    }

    fn visit_unit<E>(self) -> Result<T, E>
    where
        E: de::Error,
    {
        Err(E::custom(invalid_type_message("unit value", T::TYPE_NAME)))
    }

    fn visit_none<E>(self) -> Result<T, E>
    where
        E: de::Error,
    {
        Err(E::custom(invalid_type_message("unit value", T::TYPE_NAME)))
    }

    fn visit_seq<A>(self, _seq: A) -> Result<T, A::Error>
    where
        A: SeqAccess<'de>,
    {
        Err(de::Error::custom(invalid_type_message(
            "sequence",
            T::TYPE_NAME,
        )))
    }

    fn visit_map<A>(self, _map: A) -> Result<T, A::Error>
    where
        A: MapAccess<'de>,
    {
        Err(de::Error::custom(invalid_type_message("map", T::TYPE_NAME)))
    }
}

/// Visitor for the `strict_*_opt` helpers: `null` becomes `None`; everything
/// else delegates to [`StrictIntVisitor`] wrapped in `Some`.
#[derive(Default)]
struct StrictIntOptVisitor<T: StrictInt> {
    inner: StrictIntVisitor<T>,
}

impl<'de, T: StrictInt + 'de> Visitor<'de> for StrictIntOptVisitor<T> {
    type Value = Option<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(T::TYPE_NAME)
    }

    fn visit_u64<E>(self, value: u64) -> Result<Option<T>, E>
    where
        E: de::Error,
    {
        self.inner.visit_u64(value).map(Some)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Option<T>, E>
    where
        E: de::Error,
    {
        self.inner.visit_i64(value).map(Some)
    }

    fn visit_f64<E>(self, value: f64) -> Result<Option<T>, E>
    where
        E: de::Error,
    {
        self.inner.visit_f64(value).map(Some)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Option<T>, E>
    where
        E: de::Error,
    {
        self.inner.visit_bool(value).map(Some)
    }

    fn visit_str<E>(self, value: &str) -> Result<Option<T>, E>
    where
        E: de::Error,
    {
        self.inner.visit_str(value).map(Some)
    }

    fn visit_bytes<E>(self, value: &[u8]) -> Result<Option<T>, E>
    where
        E: de::Error,
    {
        self.inner.visit_bytes(value).map(Some)
    }

    fn visit_unit<E>(self) -> Result<Option<T>, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_none<E>(self) -> Result<Option<T>, E>
    where
        E: de::Error,
    {
        Ok(None)
    }

    fn visit_seq<A>(self, seq: A) -> Result<Option<T>, A::Error>
    where
        A: SeqAccess<'de>,
    {
        self.inner.visit_seq(seq).map(Some)
    }

    fn visit_map<A>(self, map: A) -> Result<Option<T>, A::Error>
    where
        A: MapAccess<'de>,
    {
        self.inner.visit_map(map).map(Some)
    }
}

macro_rules! strict_int_helpers {
    {
        $(
            $plain:ident => $opt:ident => $ty:ty,
        )*
    } => {
        $(
            #[doc = concat!(
                "Strictly deserialize a JSON number — or a string containing only an integer ",
                "literal — into `",
                stringify!($ty),
                "`.\n\n",
                "JSON numbers keep serde's existing accept/reject classification for `",
                stringify!($ty),
                "`; strings must be base-10 integer literals (a leading `+` is rejected ",
                "explicitly); floats, booleans, `null`, arrays, and objects are rejected ",
                "with a teachable error.\n\n",
                "Use as `#[serde(deserialize_with = \"",
                stringify!($plain),
                "\")]` on a `",
                stringify!($ty),
                "` field (coexists with `serde(default)`).",
            )]
            pub fn $plain<'de, D>(deserializer: D) -> Result<$ty, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(StrictIntVisitor::<$ty>::default())
            }

            #[doc = concat!(
                "Optional variant of [`",
                stringify!($plain),
                "`]: JSON `null` deserializes to `None`; every other value follows the plain ",
                "visitor and is wrapped in `Some`.\n\n",
                "Use as `#[serde(deserialize_with = \"",
                stringify!($opt),
                "\")]` on an `Option<",
                stringify!($ty),
                ">` field (coexists with `serde(default)`).",
            )]
            pub fn $opt<'de, D>(deserializer: D) -> Result<Option<$ty>, D::Error>
            where
                D: Deserializer<'de>,
            {
                deserializer.deserialize_any(StrictIntOptVisitor::<$ty>::default())
            }
        )*
    };
}

strict_int_helpers! {
    strict_u64 => strict_u64_opt => u64,
    strict_usize => strict_usize_opt => usize,
    strict_i32 => strict_i32_opt => i32,
    strict_i64 => strict_i64_opt => i64,
}

#[cfg(test)]
#[path = "strict_int_tests.rs"]
mod tests;
