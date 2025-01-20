pub const SEARCH_HTML: &[u8] = include_bytes!("search.html");

pub const TEMPLATES: &[(&str, &[u8])] = &[
    (
        "search_result_tag.html",
        include_bytes!("search_result_tag.html"),
    ),
    (
        "search_result_attribute.html",
        include_bytes!("search_result_attribute.html"),
    ),
    (
        "search_result_record.html",
        include_bytes!("search_result_record.html"),
    ),
    (
        "search_result_link.html",
        include_bytes!("search_result_link.html"),
    ),
];
