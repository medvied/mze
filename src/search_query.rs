pub struct SearchQuery {
    pub text: Vec<String>,
    pub tags_all: bool,
    pub tags: Vec<String>,
    pub attributes_all: bool,
    pub attribute_keys_only: Vec<String>,
    pub attribute_values_only: Vec<String>,
    pub attributes: Vec<(String, String)>,
}

impl SearchQuery {
    pub fn new(query: &str) -> SearchQuery {
        let words: Vec<_> = query.split_whitespace().collect();
        SearchQuery {
            text: words.iter().map(|word| String::from(*word)).collect(),
            tags_all: words.iter().any(|word| *word == "#"),
            tags: words
                .iter()
                .filter_map(|word| {
                    Some(String::from(Self::get_if_hashtag(word)?))
                })
                .collect(),
            attributes_all: words.iter().any(|word| *word == "#="),
            attribute_keys_only: words
                .iter()
                .filter_map(|word| {
                    let (key, value) = Self::get_if_hash_key_value(word)?;
                    if !key.is_empty() && value.is_empty() {
                        Some(String::from(key))
                    } else {
                        None
                    }
                })
                .collect(),
            attribute_values_only: words
                .iter()
                .filter_map(|word| {
                    let (key, value) = Self::get_if_hash_key_value(word)?;
                    if key.is_empty() && !value.is_empty() {
                        Some(String::from(value))
                    } else {
                        None
                    }
                })
                .collect(),
            attributes: words
                .iter()
                .filter_map(|word| {
                    let (key, value) = Self::get_if_hash_key_value(word)?;
                    if !key.is_empty() && !value.is_empty() {
                        Some((String::from(key), String::from(value)))
                    } else {
                        None
                    }
                })
                .collect(),
        }
    }

    fn get_if_hashtag(word: &str) -> Option<&str> {
        // >= 2 to avoid returning strings
        if word.len() >= 2 && word.starts_with("#") {
            Some(&word[1..])
        } else {
            None
        }
    }

    fn get_if_hash_key_value(word: &str) -> Option<(&str, &str)> {
        if word.starts_with("#") {
            word.find('=')
                .map(|pos| (&word[1..pos], &word[(pos + 1)..]))
        } else {
            None
        }
    }
}
