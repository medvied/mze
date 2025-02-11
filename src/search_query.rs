#[derive(Debug)]
pub struct SearchQueryTags {
    pub tag_substrings: Vec<String>,
}

#[derive(Debug)]
pub struct SearchQueryAttributes {
    pub attribute_key_substrings: Vec<String>,
    pub attribute_value_substrings: Vec<String>,
    pub attribute_kv_substrings: Vec<(String, String)>,
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
            let mut attribute_key_substrings = Vec::new();
            let mut attribute_value_substrings = Vec::new();
            let mut attribute_kv_substrings = Vec::new();
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
                        attribute_key_substrings.push(k);
                    } else if k.is_empty() {
                        attribute_value_substrings.push(v);
                    } else {
                        attribute_kv_substrings.push((k, v));
                    }
                } else {
                    attribute_key_substrings.push(String::from(word));
                    attribute_value_substrings.push(String::from(word));
                };
            }
            SearchQuery::Attributes(SearchQueryAttributes {
                attribute_key_substrings,
                attribute_value_substrings,
                attribute_kv_substrings,
            })
        } else {
            let mut tag_substrings = Vec::new();
            let mut attribute_key_substrings = Vec::new();
            let mut attribute_value_substrings = Vec::new();
            let mut attribute_kv_substrings = Vec::new();
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
                            attribute_key_substrings.push(k);
                        } else if k.is_empty() {
                            attribute_value_substrings.push(v);
                        } else {
                            attribute_kv_substrings.push((k, v));
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
                    attribute_key_substrings,
                    attribute_value_substrings,
                    attribute_kv_substrings,
                },
                text_substrings,
            })
        }
    }
}
