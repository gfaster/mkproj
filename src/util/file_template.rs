use std::borrow::Cow;

use std::fmt;
use std::{collections::HashMap, fmt::Display};

use anyhow::{bail, Result};

struct Replacement<'a> {
    name: &'a str,
    replacement: &'a dyn Display,
}

/// Templates are as follows:
/// ```txt
/// "{{template_name}}"
/// ```
/// The max template name length is defined by [`Template::MAX_NAME_LEN`]
pub struct Template<'a> {
    /// (text, template_key), empty for last if no trailing template
    template: Vec<(&'a str, &'a str)>,

    replacement: HashMap<&'a str, Option<Box<dyn Display + 'a>>>,
    min_len: usize,
}

impl<'a> Template<'a> {
    const MAX_NAME_LEN: usize = 30;

    pub fn new(template: &'a str) -> Result<Self> {
        const OPEN: &str = "{{";
        const CLOSE: &str = "}}";
        let mut idx = 0;
        let mut min_len = 0;
        let mut replacement = HashMap::new();
        let mut tvec = Vec::new();
        while let Some(template_start) = template[idx..].find(OPEN) {
            let text = &template[idx..(idx + template_start)];
            min_len += text.len();
            idx += template_start;
            let Some(close_off) = template[(idx + OPEN.len())..].find(CLOSE) else {
                let preview_window = template.len() - idx;
                const PREVIEW_LEN: usize = 10;
                if preview_window < PREVIEW_LEN {
                    bail!("Template brackets at the end of the template string are unclosed")
                } else {
                    bail!("Template brackets starting at {} are unclosed", &template[idx..(idx + PREVIEW_LEN)])
                }
            };
            let key = &template[(idx + OPEN.len())..(idx + OPEN.len() + close_off)];
            if key.len() > Self::MAX_NAME_LEN {
                let truncated = truncate_str(key, Self::MAX_NAME_LEN * 2);
                bail!("Template key `{truncated}` is too long ({} > {})", key.len(), Self::MAX_NAME_LEN)
            }
            if key.is_empty() {
                bail!("Template key is empty")
            }
            if key.as_bytes()[0].is_ascii_digit() || key == "_" || key.bytes().any(|b| b != b'_' && !b.is_ascii_alphanumeric()) {
                bail!("Template key `{key}` is an invalid identifier")
            }
            tvec.push((text, key));
            replacement.insert(key, None);
            idx += OPEN.len() + CLOSE.len() + close_off;
        }
        if idx != template.len() {
            let text = &template[idx..];
            min_len += text.len();
            tvec.push((text, ""));
        }
        Ok(Template { template: tvec, replacement, min_len })
    }

    pub fn subst(&mut self, key: &'a str, val: impl Display + 'a) -> &mut Self {
        let Some(old) = self.replacement.insert(key, Some(Box::new(val))) else {
            // panic!("Key `{key}` does not exist in template string")
            return self
        };
        if old.is_some() {
            panic!("Key `{key}` has multiple replacements")
        }
        self
    }

    pub fn format(&self) -> String {
        let mut buf = String::with_capacity(self.min_len);
        self.format_into(&mut buf).unwrap();
        buf
    }

    pub fn format_into(&self, buf: &mut dyn std::fmt::Write) -> std::fmt::Result {
        for &(text, key) in &self.template {
            buf.write_str(text)?;
            if !key.is_empty() {
                // key should always be present, but user error can lead to it being unset
                let Some(replace) = &self.replacement[key] else {
                    // panic!("Template requires key `{key}` to be specified")
                    continue
                };
                write!(buf, "{}", replace)?;
            }
        }
        Ok(())
    }
}

impl fmt::Debug for Template<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &(text, key) in &self.template {
            f.write_str(text)?;
            if !key.is_empty() {
                // key should always be present in the map
                if let Some(replace) = &self.replacement[key] {
                    write!(f, "{}", replace)?;
                } else {
                    write!(f, "<{key}: MISSING>")?;
                };
            }
        }
        Ok(())
    }
}

// fn display_from_

/// shortens `s` if it's longer than `len`, but does not make any hard guarantees about it's
/// length, nor that it makes it shorter for small values of `len`
#[must_use]
fn truncate_str(s: &str, len: usize) -> impl Display + use<'_> {
    // TODO: I can make this not allocate at all with a custom struct or const generic len
    if s.len() <= len {
        return Cow::Borrowed(s)
    }
    let boundary = floor_char_boundary(s, len / 3);
    let split_first = &s[..boundary];
    let boundary = floor_char_boundary(s, s.len() - len / 3);
    let split_second = &s[boundary..];
    format!("{split_first}...{split_second}").into()
}

/// rounds down `pos` to a valid character boundary
///
/// This is just a copy of the unstable function by the same name, and should be replaced when it's
/// stabilized
/// ```txt
/// #[unstable(feature = "round_char_boundary", issue = "93743")]
/// ```
fn floor_char_boundary(s: &str, index: usize) -> usize {
    fn is_utf8_char_boundary(byte: u8) -> bool {
        // This is bit magic equivalent to: b < 128 || b >= 192
        (byte as i8) >= -0x40
    }
    if index >= s.len() {
        s.len()
    } else {
        let lower_bound = index.saturating_sub(3);
        let new_index = s.as_bytes()[lower_bound..=index]
            .iter()
            .rposition(|b| is_utf8_char_boundary(*b));

        // this is an `unwraped_unchecked` in std, but I'm not about to introduce unnecessary
        // unsafe
        lower_bound + new_index.unwrap()
    }
}

macro_rules! template_format {
    ($template:expr $(, $replacee:ident = $replacement:expr)*$(,)?) => {
        {
        $(let ref $replacee = $replacement;)*
        #[allow(unused_mut)]
        let mut template = $crate::util::file_template::Template::new(&$template).unwrap();
        $(template.subst(stringify!($replacee), $replacee);)*
        template.format()
        }
    };
}
#[allow(unused_imports)]
pub(crate) use template_format;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncatation() {
        // from str::is_char_boundary docs
        let s = "Löwe 老虎 Léopard";
        assert_eq!(truncate_str(s, s.len()).to_string(), s);

        let truncated = truncate_str(s, s.len() / 2).to_string();

        assert!(truncated.len() < s.len());
        let split_pos = truncated.find("...").expect("truncation had no ellipses");
        assert_eq!(&truncated[..split_pos], &s[..split_pos]);
        assert_eq!(&truncated[split_pos..split_pos + 3], "...");
        assert!(s.ends_with(&truncated[3 + split_pos..]));
    }

    #[test]
    fn truncate_str_nopanic() {
        // from str::is_char_boundary docs
        let s = "Löwe 老虎 Léopard";
        for len in 0..s.len() + 5 {
            let _ = truncate_str(s, len);
        }
    }

    #[test]
    fn template_invalid_key_fails() {
        let err = Template::new("{{this_key_is_very_very_very_very_very_very_very_very_very_long}}").unwrap_err();
        assert!(format!("{err:#?}").contains("is too long"));

        let err = Template::new("{{hello-world}}").unwrap_err();
        assert!(format!("{err:#?}").contains("is an invalid identifier"));

        let err = Template::new("{{hello world}}").unwrap_err();
        assert!(format!("{err:#?}").contains("is an invalid identifier"));

        let err = Template::new("{{9front}}").unwrap_err();
        assert!(format!("{err:#?}").contains("is an invalid identifier"));

        let err = Template::new("{{こんにちは}}").unwrap_err();
        assert!(format!("{err:#?}").contains("is an invalid identifier"));

        let err = Template::new("{{_}}").unwrap_err();
        assert!(format!("{err:#?}").contains("is an invalid identifier"));

        let err = Template::new("{{}}").unwrap_err();
        assert!(format!("{err:#?}").contains("is empty"));
    }

    #[test]
    fn template_valid_key_passes() {
        Template::new("{{basic}}").unwrap();
        Template::new("{{hello_world}}").unwrap();
        Template::new("{{_leading_underscore}}").unwrap();
        Template::new("{{plan9}}").unwrap();
        Template::new("{{ALL_CAPS}}").unwrap();
    }

    #[test]
    fn template_replacements() {
        let actual = Template::new("hello, world").unwrap().format();
        assert_eq!(actual, "hello, world");

        let actual = Template::new("{{all_template}}").unwrap().subst("all_template", &"hello, world").format();
        assert_eq!(actual, "hello, world");

        let actual = Template::new("{{start}}, {{end}}").unwrap().subst("start", &"hello").subst("end", &"world").format();
        assert_eq!(actual, "hello, world");

        let actual = Template::new("{{dup}} + {{dup}}").unwrap().subst("dup", &"0xFF").format();
        assert_eq!(actual, "0xFF + 0xFF");

        let actual = Template::new("{{one}}, {{two}}, {{three}}").unwrap().subst("one", &1).subst("two", &2).subst("three", &3).format();
        assert_eq!(actual, "1, 2, 3");

        let actual = Template::new("start, {{step2}}, profit").unwrap().subst("step2", &"???").format();
        assert_eq!(actual, "start, ???, profit");
    }
}
