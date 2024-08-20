use std::collections::HashSet;

pub fn lowercase() -> HashSet<String> {
    let conjunctions = vec!["and", "but", "for", "nor", "or", "so", "yet"];
    let articles = vec!["a", "an", "the"];
    let prepositions = vec![
        "about",
        "above",
        "across",
        "after",
        "against",
        "along",
        "among",
        "around",
        "at",
        "before",
        "behind",
        "below",
        "beneath",
        "beside",
        "between",
        "beyond",
        "by",
        "concerning",
        "considering",
        "despite",
        "down",
        "during",
        "except",
        "excepting",
        "excluding",
        "following",
        "for",
        "from",
        "in",
        "inside",
        "into",
        "like",
        "minus",
        "near",
        "of",
        "off",
        "on",
        "onto",
        "opposite",
        "over",
        "past",
        "per",
        "plus",
        "regarding",
        "round",
        "save",
        "since",
        "than",
        "through",
        "to",
        "toward",
        "towards",
        "under",
        "underneath",
        "unlike",
        "until",
        "up",
        "upon",
        "versus",
        "via",
        "with",
        "within",
        "without",
    ];

    let mut combined_set = HashSet::new();
    combined_set.extend(conjunctions.iter().map(|s| s.to_string()));
    combined_set.extend(articles.iter().map(|s| s.to_string()));
    combined_set.extend(prepositions.iter().map(|s| s.to_string()));

    combined_set
}
