mod lower_case;
mod specials;

use regex::{Captures, Regex, RegexBuilder};

static WORD: &str = r#"[^\s'’()!?;:\"-]"#;

fn convert_to_regex(specials: Vec<String>) -> Vec<(Regex, String)> {
    specials
        .iter()
        .map(|s| {
            (
                RegexBuilder::new(&format!(r"\b{}\b", regex::escape(s)))
                    .case_insensitive(true)
                    .build()
                    .unwrap(),
                s.to_string(),
            )
        })
        .collect()
}

fn parse_match(match_str: &str) -> Option<&str> {
    let first_character = match_str.chars().next()?;

    if first_character.is_whitespace() {
        return Some(&match_str[1..]);
    }
    if ['(', ')'].contains(&first_character) {
        return None;
    }

    Some(match_str)
}

pub fn title_case(str: &str, custom_sepcials: Option<Vec<&str>>) -> String {
    let mut result = str.to_lowercase();
    let lower_case_set = lower_case::lowercase();

    let regex = Regex::new(&format!(
        r#"(?:(?:(\s?(?:^|[.()!?;:"-])\s*)({}))|({}))({}*['’]*{}*)"#,
        WORD, WORD, WORD, WORD
    ))
    .unwrap();

    result = regex
        .replace_all(&result, |caps: &Captures| {
            let m = caps.get(0).unwrap().as_str();
            let lead = caps.get(1).map_or("", |m| m.as_str());
            let forced = caps.get(2).map_or("", |m| m.as_str());
            let lower = caps.get(3).map_or("", |m| m.as_str());
            let rest = caps.get(4).map_or("", |m| m.as_str());
            let offset = caps.get(0).unwrap().start();
            let is_last_word = m.len() + offset >= str.len();

            let parsed_match = parse_match(m);

            if let None = parsed_match {
                return m.to_string();
            }

            if forced.is_empty() {
                let full_lower = format!("{}{}", lower, rest);

                if lower_case_set.contains(&full_lower) && !is_last_word {
                    return parsed_match.unwrap().to_string();
                }
            }

            let lower_or_forced = if lower.is_empty() { forced } else { lower };

            format!("{lead}{}{rest}", lower_or_forced.to_uppercase()).to_string()
        })
        .to_string();

    let custom_specials = custom_sepcials.unwrap_or(vec![]);
    let replace = [&specials::INTENDED[..], &custom_specials].concat();
    let replace_regex = convert_to_regex(replace.iter().map(|s| s.to_string()).collect());

    for (pattern, s) in replace_regex {
        result = pattern.replace_all(&result, &*s).to_string();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() {
        assert_eq!(
            title_case("updates TO hAndLinG of Failed paYMEnts", None),
            "Updates to Handling of Failed Payments"
        );
        assert_eq!(
            title_case("capitalize your titles", None),
            "Capitalize Your Titles"
        );
        assert_eq!(
            title_case("seattle’S BEST coffee & grandma's cookies", None),
            "Seattle’s Best Coffee & Grandma's Cookies"
        );
    }

    #[test]
    fn with_default_special_cases() {
        assert_eq!(
            title_case("toWArds NEXT.JS 5: Introducing cANaRY Updates", None),
            "Towards Next.js 5: Introducing Canary Updates"
        );
        assert_eq!(
            title_case("aPi 2.0: lOG-in with zeit, new dOCs & more", None),
            "API 2.0: Log-In with ZEIT, New Docs & More"
        );
        assert_eq!(
            title_case("noW deSktop and now cLI are prODUCts of zeIt", None),
            "Now Desktop and Now CLI Are Products of ZEIT"
        );
    }

    #[test]
    fn with_custom_special_cases() {
        assert_eq!(
            title_case(
                "mY cusToM brand is awesome",
                vec!["BRAnD", "awesoMe"].into()
            ),
            "My Custom BRAnD Is awesoMe"
        );
        assert_eq!(
            title_case(
                "modify speCials like Facebook or microsoft",
                Some(vec!["facebook", "Microsoft"])
            ),
            "Modify Specials like facebook or Microsoft"
        );
    }

    #[test]
    fn parenthesis() {
        let from = "employment region(s) for my application";
        let to = "Employment Region(s) for My Application";
        assert_eq!(title_case(from, None), to);

        let from = "(s)omething or other";
        let to = "(s)omething or Other";
        assert_eq!(title_case(from, None), to);

        let from = "cat(s) can be a pain";
        let to = "Cat(s) can Be a Pain";
        assert_eq!(title_case(from, None), to);
    }

    #[test]
    fn keep_lowercased() {
        let from = "there and beyond";
        let to = "There and Beyond";
        assert_eq!(title_case(from, None), to);

        let from = "be careful what you wish for";
        let to = "Be Careful What You Wish For";
        assert_eq!(title_case(from, None), to);

        let from = "XYZ: what is it good for";
        let to = "XYZ: What Is It Good For";
        assert_eq!(title_case(from, Some(vec!["XYZ"])), to);
    }

    #[test]
    fn non_english_words() {
        let from = "çeşme city";
        let to = "Çeşme City";
        assert_eq!(title_case(from, None), to);

        let from = "la niña esta aquí";
        let to = "La Niña Esta Aquí";
        assert_eq!(title_case(from, None), to);

        let from = "forhandlingsmøde";
        let to = "Forhandlingsmøde";
        assert_eq!(title_case(from, None), to);

        let from = "đội";
        let to = "Đội";
        assert_eq!(title_case(from, None), to);

        let from = "tuyển";
        let to = "Tuyển";
        assert_eq!(title_case(from, None), to);
    }
}
