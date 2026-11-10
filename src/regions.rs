use crate::tokenize::paragraphs;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PassageRole {
    Prose,
    Quotation,
    Bibliography,
}

/// Structural labels for blank-line paragraphs, in the same order as `paragraphs`.
pub(crate) fn paragraph_roles(text: &str) -> Vec<PassageRole> {
    let paras = paragraphs(text);
    let mut roles = Vec::with_capacity(paras.len());
    let mut in_bibliography = false;
    for paragraph in paras {
        match heading_kind(&paragraph) {
            Some(HeadingKind::Bibliography) => {
                in_bibliography = true;
                roles.push(PassageRole::Bibliography);
                continue;
            }
            Some(HeadingKind::Other) => {
                in_bibliography = false;
                roles.push(PassageRole::Prose);
                continue;
            }
            None => {}
        }
        if in_bibliography {
            roles.push(PassageRole::Bibliography);
        } else if is_quotation(&paragraph) {
            roles.push(PassageRole::Quotation);
        } else {
            roles.push(PassageRole::Prose);
        }
    }
    roles
}

enum HeadingKind {
    Bibliography,
    Other,
}

fn heading_kind(paragraph: &str) -> Option<HeadingKind> {
    let trimmed = paragraph.trim();
    let core = trimmed
        .trim_start_matches('#')
        .trim()
        .trim_matches('*')
        .trim();
    if core.chars().count() > 80 {
        return None;
    }
    let lower = core.to_lowercase();
    const BIB: &[&str] = &[
        "references",
        "bibliography",
        "works cited",
        "reference list",
        "参考文献",
        "引用文献",
        "参考资料",
    ];
    if BIB.iter().any(|heading| {
        lower == *heading
            || lower.starts_with(&format!("{heading}:"))
            || lower.starts_with(&format!("{heading}："))
    }) {
        return Some(HeadingKind::Bibliography);
    }
    if trimmed.starts_with('#') {
        return Some(HeadingKind::Other);
    }
    const SECTIONS: &[&str] = &[
        "introduction",
        "method",
        "methods",
        "results",
        "discussion",
        "conclusion",
        "data",
        "引言",
        "步骤",
        "数据",
        "说明",
        "讨论",
        "结论",
    ];
    if SECTIONS.contains(&lower.as_str()) {
        return Some(HeadingKind::Other);
    }
    None
}

fn is_quotation(paragraph: &str) -> bool {
    let trimmed = paragraph.trim();
    if trimmed.starts_with('>') {
        return true;
    }
    if wrapped_in_quotes(trimmed) && content_chars(trimmed) >= 40 {
        return true;
    }
    let mut quoted = 0usize;
    let mut total = 0usize;
    let mut depth_straight = false;
    let mut depth_curly = 0i32;
    let mut depth_corner = 0i32;
    let mut depth_white = 0i32;
    for ch in trimmed.chars() {
        match ch {
            '"' => depth_straight = !depth_straight,
            '“' => depth_curly += 1,
            '”' => depth_curly = (depth_curly - 1).max(0),
            '「' => depth_corner += 1,
            '」' => depth_corner = (depth_corner - 1).max(0),
            '『' => depth_white += 1,
            '』' => depth_white = (depth_white - 1).max(0),
            _ => {}
        }
        if is_content(ch) {
            total += 1;
            if depth_straight || depth_curly > 0 || depth_corner > 0 || depth_white > 0 {
                quoted += 1;
            }
        }
    }
    total >= 40 && quoted * 2 >= total
}

fn wrapped_in_quotes(text: &str) -> bool {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() < 2 {
        return false;
    }
    matches!(
        (chars[0], chars[chars.len() - 1]),
        ('"', '"') | ('“', '”') | ('「', '」') | ('『', '』')
    )
}

fn content_chars(text: &str) -> usize {
    text.chars().filter(|ch| is_content(*ch)).count()
}

fn is_content(ch: char) -> bool {
    ch.is_alphanumeric() || is_han(ch)
}

fn is_han(ch: char) -> bool {
    matches!(ch, '\u{3400}'..='\u{9FFF}')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bibliography_and_quotes_are_labeled_without_a_verdict() {
        let text = "\
Synthetic fixture.

The group heated a beaker of water and recorded the cooling on the bench for this prose paragraph which is long enough to stay in the main comparison.

\"In this synthetic cooling exercise the group recorded the temperature of two hundred millilitres of water and the later readings fell steadily over several minutes.\"

# References

Synthetic Author. A made up cooling note for a fixture bibliography entry that is long enough to form several four token shingles in the reference list.

# Discussion

After the references heading the discussion is prose again and it is long enough to be compared as the student's own paragraph.
";
        let roles = paragraph_roles(text);
        assert!(roles.contains(&PassageRole::Quotation));
        assert!(roles.contains(&PassageRole::Bibliography));
        assert!(roles.contains(&PassageRole::Prose));
        let bib_at = roles
            .iter()
            .position(|role| *role == PassageRole::Bibliography)
            .unwrap();
        assert!(roles[bib_at + 1] == PassageRole::Bibliography);
        assert_eq!(*roles.last().unwrap(), PassageRole::Prose);
    }
}
