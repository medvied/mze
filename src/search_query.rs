#[derive(Debug)]
pub struct SearchQueryTags {
    pub tag_substrings: Vec<String>,
}

#[derive(Debug)]
pub struct SearchQueryAttributes {
    pub key_substrings: Vec<String>,
    pub value_substrings: Vec<String>,
    pub kv_substrings: Vec<(String, String)>,
}

#[derive(Debug)]
pub struct SearchQueryRecordsAndLinks {
    pub tags: SearchQueryTags,
    pub attributes: SearchQueryAttributes,
    pub text_substrings: Vec<String>,
}

#[derive(Debug)]
pub enum SearchQuery {
    Tags(SearchQueryTags),
    Attributes(SearchQueryAttributes),
    RecordsAndLinks(SearchQueryRecordsAndLinks),
}

impl SearchQueryTags {
    pub fn is_empty(&self) -> bool {
        self.tag_substrings.is_empty()
    }

    pub fn check(&self, tag: &str) -> bool {
        self.tag_substrings.iter().any(|t| tag.contains(t))
    }
}

impl SearchQueryAttributes {
    pub fn is_empty(&self) -> bool {
        self.key_substrings.is_empty()
            && self.value_substrings.is_empty()
            && self.kv_substrings.is_empty()
    }

    pub fn check(&self, key: &str, value: &str) -> bool {
        self.kv_substrings
            .iter()
            .any(|(k, v)| key.contains(k) && value.contains(v))
            || self.key_substrings.iter().any(|k| key.contains(k))
            || self.value_substrings.iter().any(|v| value.contains(v))
    }
}

impl SearchQuery {
    pub fn new(query: &str) -> SearchQuery {
        let words: Vec<_> = query.split_whitespace().collect();
        if words.contains(&"#") {
            let tag_substrings = words
                .into_iter()
                .filter_map(|word| {
                    if word.is_empty() || word == "#" {
                        return None;
                    }
                    Some(String::from(
                        if let Some(stripped) = word.strip_prefix("#") {
                            stripped
                        } else {
                            word
                        },
                    ))
                })
                .collect();
            SearchQuery::Tags(SearchQueryTags { tag_substrings })
        } else if words.contains(&"#=") {
            let mut key_substrings = Vec::new();
            let mut value_substrings = Vec::new();
            let mut kv_substrings = Vec::new();
            for word in words {
                let word = if let Some(stripped) = word.strip_prefix("#") {
                    stripped
                } else {
                    word
                };
                if word.is_empty() || word == "=" {
                    continue;
                }
                if let Some(equals_pos) = word.find("=") {
                    let k = String::from(&word[..equals_pos]);
                    let v = String::from(&word[equals_pos + 1..]);
                    if v.is_empty() {
                        assert!(!k.is_empty());
                        key_substrings.push(k);
                    } else if k.is_empty() {
                        value_substrings.push(v);
                    } else {
                        kv_substrings.push((k, v));
                    }
                } else {
                    key_substrings.push(String::from(word));
                    value_substrings.push(String::from(word));
                };
            }
            SearchQuery::Attributes(SearchQueryAttributes {
                key_substrings,
                value_substrings,
                kv_substrings,
            })
        } else {
            let mut tag_substrings = Vec::new();
            let mut key_substrings = Vec::new();
            let mut value_substrings = Vec::new();
            let mut kv_substrings = Vec::new();
            let mut text_substrings = Vec::new();

            for word in words {
                assert!(!word.is_empty());
                assert!(word != "#");
                assert!(word != "#=");
                if let Some(word) = word.strip_prefix("#") {
                    if let Some(equals_pos) = word.find("=") {
                        let k = String::from(&word[..equals_pos]);
                        let v = String::from(&word[equals_pos + 1..]);
                        // XXX copy-paste
                        if v.is_empty() {
                            assert!(!k.is_empty());
                            key_substrings.push(k);
                        } else if k.is_empty() {
                            value_substrings.push(v);
                        } else {
                            kv_substrings.push((k, v));
                        }
                    } else {
                        tag_substrings.push(String::from(word));
                    }
                } else {
                    text_substrings.push(String::from(word));
                }
            }

            SearchQuery::RecordsAndLinks(SearchQueryRecordsAndLinks {
                tags: SearchQueryTags { tag_substrings },
                attributes: SearchQueryAttributes {
                    key_substrings,
                    value_substrings,
                    kv_substrings,
                },
                text_substrings,
            })
        }
    }
}
