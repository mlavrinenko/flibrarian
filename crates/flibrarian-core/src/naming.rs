const MAX_FILE_NAME_LEN: usize = 255;
const TRUNCATION_MARKER: &str = "...";

pub fn normalized_author(author: Option<&str>) -> String {
    author.unwrap_or_default().trim().replace(" ,", ", ")
}

pub fn book_file_name(author: &str, title: &str, book_id: u32) -> String {
    let suffix = format!(" [{book_id}].fb2");
    let budget = MAX_FILE_NAME_LEN.saturating_sub(suffix.len());
    let stem = if author.is_empty() {
        truncate(title, budget)
    } else {
        book_stem(author, title, budget)
    };
    format!("{stem}{suffix}")
}

fn book_stem(author: &str, title: &str, budget: usize) -> String {
    const SEPARATOR: &str = " - ";
    let title_budget = budget.saturating_sub(SEPARATOR.len());

    if title.len() <= title_budget {
        let author = truncate(author, title_budget - title.len());
        return join_stem(&author, title);
    }

    let author_share = author.len().min(budget / 2);
    let title = truncate(title, title_budget - author_share);
    let author = truncate(author, budget - SEPARATOR.len() - title.len());
    join_stem(&author, &title)
}

fn join_stem(author: &str, title: &str) -> String {
    if author.is_empty() {
        title.to_string()
    } else {
        format!("{author} - {title}")
    }
}

fn truncate(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    let Some(limit) = max_len.checked_sub(TRUNCATION_MARKER.len()) else {
        return String::new();
    };
    let cut = text
        .char_indices()
        .take_while(|(index, _)| *index <= limit)
        .map(|(index, _)| index)
        .last()
        .unwrap_or(0);
    let truncated = text[..cut].trim_end_matches([' ', ',']);
    format!("{truncated}{TRUNCATION_MARKER}")
}
