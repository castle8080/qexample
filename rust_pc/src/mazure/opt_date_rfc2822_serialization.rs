use std::fmt;
use serde::{Serializer, Deserializer, de};
use serde::de::Visitor;
use chrono::{DateTime, Utc};

struct DateTimeVisitor;

impl<'de> Visitor<'de> for DateTimeVisitor {
    type Value = DateTime<Utc>;
    
    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an rfc2822 date string.")
    }

    fn visit_str<E: de::Error>(self, s: &str) -> Result<Self::Value, E> {
        Ok(DateTime::parse_from_rfc2822(s).map_err(de::Error::custom)?.with_timezone(&Utc))
    }
}

struct OptDateTimeVisitor;

impl<'de> Visitor<'de> for OptDateTimeVisitor {
    type Value = Option<DateTime<Utc>>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("an rfc2822 date string.")
    }

    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_str(DateTimeVisitor).map(Some)
    }
    
    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(None)
    }
}

pub fn serialize<S: Serializer>(dt: &Option<DateTime<Utc>>, serializer: S) -> Result<S::Ok, S::Error> {
    match dt {
        None => serializer.serialize_none(),
        Some(dt) => {
            let s = dt.to_rfc2822();
            serializer.serialize_str(&s)
        }
    }
}

pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Option<DateTime<Utc>>, D::Error>
{
    deserializer.deserialize_option(OptDateTimeVisitor)
}