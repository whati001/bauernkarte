//! Forgiving text search for the product picker and the store-name
//! filter: case, word order, umlaut spelling (Äpfel = Aepfel = Apfel),
//! accents and a typo or two don't stop a match.

/// Whether every word of `query` turns up in some word of `text`, in any
/// order ("eier karner" = "karner eier"). A query word may be just part
/// of a text word ("saft" in "Apfelsaft"). Longer words may be off by a
/// typo (two for long ones): a wrong, missing, extra or swapped letter.
/// An empty query matches everything.
pub fn matches(query: &str, text: &str) -> bool {
    let text = fold(text);
    let tokens: Vec<Vec<char>> = words(&text).map(|t| t.chars().collect()).collect();
    words(&fold(query)).all(|word| {
        let word: Vec<char> = word.chars().collect();
        let typos = allowed_typos(word.len());
        tokens.iter().any(|token| contains_within(&word, token, typos))
    })
}

fn words(s: &str) -> impl Iterator<Item = &str> {
    s.split(' ').filter(|w| !w.is_empty())
}

/// Short words must match exactly: one typo in three letters matches
/// nearly anything.
fn allowed_typos(len: usize) -> usize {
    match len {
        0..=4 => 0,
        5..=8 => 1,
        _ => 2,
    }
}

/// Lowercase, accents and umlauts reduced to the base letter, and the
/// "ae"/"oe"/"ue" spellings of umlauts reduced the same way. Anything
/// that isn't a letter or digit separates words.
fn fold(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars().flat_map(char::to_lowercase) {
        match c {
            'ä' | 'à' | 'á' | 'â' | 'ã' | 'å' => out.push('a'),
            'ö' | 'ò' | 'ó' | 'ô' | 'õ' | 'ø' => out.push('o'),
            'ü' | 'ù' | 'ú' | 'û' => out.push('u'),
            'è' | 'é' | 'ê' | 'ë' => out.push('e'),
            'ì' | 'í' | 'î' | 'ï' => out.push('i'),
            'ç' => out.push('c'),
            'ñ' => out.push('n'),
            'ß' => out.push_str("ss"),
            // Only drops the "e", so a query cut off just before it
            // ("qu" for "Quelle") still matches.
            'e' if matches!(out.chars().last(), Some('a' | 'o' | 'u')) => {}
            c if c.is_alphanumeric() => out.push(c),
            _ => out.push(' '),
        }
    }
    out
}

/// Whether `pattern` occurs in `text` with at most `max` edits
/// (substitution, insertion, deletion or swap of neighbours). Sellers'
/// variant of the edit-distance table: the match may start anywhere in
/// `text`, so row 0 is all zeros, and end anywhere, so the last row's
/// minimum counts.
fn contains_within(pattern: &[char], text: &[char], max: usize) -> bool {
    let n = text.len();
    let mut before = vec![0; n + 1];
    let mut prev = vec![0; n + 1];
    for i in 1..=pattern.len() {
        let mut cur = vec![i; n + 1];
        for j in 1..=n {
            let cost = usize::from(pattern[i - 1] != text[j - 1]);
            let mut d = (prev[j - 1] + cost).min(prev[j] + 1).min(cur[j - 1] + 1);
            if i > 1 && j > 1 && pattern[i - 1] == text[j - 2] && pattern[i - 2] == text[j - 1] {
                d = d.min(before[j - 2] + 1);
            }
            cur[j] = d;
        }
        before = std::mem::replace(&mut prev, cur);
    }
    prev.iter().any(|&d| d <= max)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn umlaut_spellings_are_equal() {
        assert!(matches("apfel", "Äpfel"));
        assert!(matches("Aepfel", "Äpfel"));
        assert!(matches("äpfel", "Apfelsaft"));
        assert!(matches("kase", "Käse"));
        assert!(matches("strasse", "Hofstraße"));
        assert!(matches("creme", "Crème fraîche"));
    }

    #[test]
    fn partial_words_match() {
        assert!(matches("äpf", "Äpfel"));
        assert!(matches("qu", "Quelle"));
        assert!(matches("saft", "Apfelsaft"));
    }

    #[test]
    fn longer_words_forgive_typos() {
        assert!(matches("apfle", "Äpfel"));
        assert!(matches("kartofel", "Kartoffeln"));
        assert!(matches("erdbere", "Erdbeeren"));
        assert!(matches("marmelaede", "Marillenmarmelade"));
    }

    #[test]
    fn short_words_must_be_exact() {
        assert!(!matches("bir", "Eier"));
        assert!(!matches("hase", "Käse"));
    }

    #[test]
    fn word_order_does_not_matter() {
        assert!(matches("eier karner", "Karner Eier"));
        assert!(matches("karner eier", "Karner Eier"));
        assert!(matches("hof karner", "Hofladen Karner"));
    }

    #[test]
    fn a_word_matches_within_one_name_word() {
        // "ab" + "hof" spans the space between the two words.
        assert!(!matches("abhof", "Ab Hof"));
        assert!(matches("ab hof", "Ab Hof"));
    }

    #[test]
    fn every_word_must_match() {
        assert!(matches("bio eier", "Bio-Eier"));
        assert!(!matches("bio honig", "Bio-Eier"));
    }

    #[test]
    fn empty_query_matches_everything() {
        assert!(matches("", "Äpfel"));
        assert!(matches("  ", "Äpfel"));
    }
}
