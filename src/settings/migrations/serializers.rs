pub mod color32_serde {
    use colors_transform::{Color, Rgb};
    use egui::Color32;
    use serde::{de::Visitor, Deserializer};

    #[inline(always)]
    pub fn color_parser(s: &str) -> Result<Color32, String> {
        let rgb = Rgb::from_hex_str(s).map_err(|e| e.message)?;
        Ok(Color32::from_rgb(
            rgb.get_red() as u8,
            rgb.get_green() as u8,
            rgb.get_blue() as u8,
        ))
    }

    pub struct ColorVisitor;

    impl<'de> Visitor<'de> for ColorVisitor {
        type Value = Color32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A color encoded as a hex string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            color_parser(v)
                .map_err(|e| E::invalid_value(serde::de::Unexpected::Str(v), &e.as_str()))
        }
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Color32, D::Error>
    where
        D: Deserializer<'de>,
    {
        de.deserialize_str(ColorVisitor)
    }
}

pub mod range_serde {
    use std::ops::RangeInclusive;

    use serde::{
        de::{self, Visitor},
        Deserializer,
    };

    use serde_derive::Deserialize;

    #[derive(Deserialize)]
    #[serde(field_identifier, rename_all = "lowercase")]
    enum Field {
        Hi,
        Lo,
    }

    pub struct RangeVisitor;

    impl<'de> Visitor<'de> for RangeVisitor {
        type Value = RangeInclusive<u8>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("A color encoded as a hex string")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut hi = None;
            let mut lo = None;
            while let Some(key) = map.next_key::<Field>()? {
                match key {
                    Field::Hi => {
                        if hi.is_some() {
                            return Err(de::Error::duplicate_field("hi"));
                        }
                        hi = Some(map.next_value::<u8>()?);
                    }
                    Field::Lo => {
                        if lo.is_some() {
                            return Err(de::Error::duplicate_field("lo"));
                        }
                        lo = Some(map.next_value::<u8>()?);
                    }
                }
            }

            let hi = hi.ok_or_else(|| de::Error::missing_field("hi"))?;
            let lo = lo.ok_or_else(|| de::Error::missing_field("lo"))?;

            Ok(hi..=lo)
        }
    }

    pub fn deserialize<'de, D>(de: D) -> Result<RangeInclusive<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        de.deserialize_struct("range", &["hi", "lo"], RangeVisitor)
    }
}
