use super::SearchFilters;

pub fn parse_filter_value(raw: &str) -> (bool, &str) {
    let trimmed = raw.trim();
    trimmed
        .strip_prefix('=')
        .map_or((false, trimmed), |exact| (true, exact))
}

/// Two-character operators first, so `>=` never parses as `>`.
const SIZE_OPS: [&str; 4] = [">=", "<=", ">", "<"];

pub fn parse_file_size_filter(raw: &str) -> Option<(&str, u64)> {
    let trimmed = raw.trim();
    let (op, rest) = SIZE_OPS
        .iter()
        .find_map(|op| trimmed.strip_prefix(op).map(|rest| (*op, rest)))?;

    let rest = rest.trim();
    let num_end = rest
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(rest.len());
    let num: f64 = rest[..num_end].parse().ok()?;
    let unit = rest[num_end..].trim().to_lowercase();

    let multiplier: f64 = match unit.as_str() {
        "" | "b" => 1.0,
        "kb" | "k" => 1024.0,
        "mb" | "m" => 1024.0 * 1024.0,
        "gb" | "g" => 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "file sizes are always positive and fit in u64"
    )]
    Some((op, (num * multiplier) as u64))
}

fn build_file_size_condition(
    raw: &str,
    table_alias: &str,
    conditions: &mut Vec<String>,
    params: &mut Vec<String>,
) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    if let Some((op, bytes)) = parse_file_size_filter(trimmed) {
        conditions.push(format!(
            "AND (SELECT book.file_size FROM books book WHERE book.id = {table_alias}.id) {op} {bytes}"
        ));
    } else {
        let (exact, v) = parse_filter_value(raw);
        if !v.is_empty() {
            conditions.push(format!(
                "AND CAST((SELECT book.file_size FROM books book WHERE book.id = {table_alias}.id) AS VARCHAR) ILIKE ?"
            ));
            params.push(if exact {
                v.to_string()
            } else {
                format!("%{v}%")
            });
        }
    }
}

fn build_lang_condition(
    raw: &str,
    table_alias: &str,
    conditions: &mut Vec<String>,
    params: &mut Vec<String>,
) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }

    if trimmed.contains('|') {
        let parts: Vec<&str> = trimmed.split('|').collect();
        let mut or_parts = Vec::new();
        for part in &parts {
            let part = part.trim();
            if part.is_empty() {
                or_parts.push(format!(
                    "(SELECT book.lang FROM books book WHERE book.id = {table_alias}.id) = ''"
                ));
            } else {
                or_parts.push(format!(
                    "(SELECT book.lang FROM books book WHERE book.id = {table_alias}.id) ILIKE ?"
                ));
                params.push(part.to_string());
            }
        }
        if !or_parts.is_empty() {
            conditions.push(format!("AND ({})", or_parts.join(" OR ")));
        }
        return;
    }

    let (exact, v) = parse_filter_value(trimmed);
    if !v.is_empty() {
        conditions.push(format!(
            "AND (SELECT book.lang FROM books book WHERE book.id = {table_alias}.id) ILIKE ?"
        ));
        params.push(if exact {
            v.to_string()
        } else {
            format!("%{v}%")
        });
    }
}

pub fn build_filter_conditions(
    filters: &SearchFilters,
    table_alias: &str,
) -> (String, Vec<String>) {
    let mut conditions = Vec::new();
    let mut params = Vec::new();

    if let Some(ref v) = filters.title {
        let (exact, v) = parse_filter_value(v);
        if !v.is_empty() {
            conditions.push(format!("AND {table_alias}.title ILIKE ?"));
            params.push(if exact {
                v.to_string()
            } else {
                format!("%{v}%")
            });
        }
    }
    if let Some(ref v) = filters.authors {
        let (exact, v) = parse_filter_value(v);
        if !v.is_empty() {
            conditions.push(format!(
                "AND {table_alias}.id IN (\
                    SELECT ba.book_id FROM books_authors ba \
                    JOIN authors a ON ba.author_id = a.id \
                    WHERE CONCAT_WS(' ', a.first_name, a.middle_name, a.last_name, a.nickname) ILIKE ?\
                )"
            ));
            params.push(if exact {
                v.to_string()
            } else {
                format!("%{v}%")
            });
        }
    }
    if let Some(ref v) = filters.genres {
        let (exact, v) = parse_filter_value(v);
        if !v.is_empty() {
            if exact {
                conditions.push(format!(
                    "AND EXISTS (SELECT 1 FROM json_each(\
                        (SELECT book.genres::JSON FROM books book WHERE book.id = {table_alias}.id)\
                    ) je WHERE json_extract_string(je.value, '$') ILIKE ?)"
                ));
                params.push(v.to_string());
            } else {
                conditions.push(format!(
                    "AND (SELECT book.genres FROM books book WHERE book.id = {table_alias}.id)::VARCHAR ILIKE ?"
                ));
                params.push(format!("%{v}%"));
            }
        }
    }
    if let Some(ref v) = filters.date {
        let (exact, v) = parse_filter_value(v);
        if !v.is_empty() {
            conditions.push(format!(
                "AND (SELECT book.date FROM books book WHERE book.id = {table_alias}.id) ILIKE ?"
            ));
            params.push(if exact {
                v.to_string()
            } else {
                format!("%{v}%")
            });
        }
    }
    if let Some(ref v) = filters.lang {
        build_lang_condition(v, table_alias, &mut conditions, &mut params);
    }
    if let Some(ref v) = filters.file_size {
        build_file_size_condition(v, table_alias, &mut conditions, &mut params);
    }
    if let Some(ref v) = filters.sequence {
        let (exact, v) = parse_filter_value(v);
        if !v.is_empty() {
            conditions.push(format!("AND {table_alias}.sequence ILIKE ?"));
            params.push(if exact {
                v.to_string()
            } else {
                format!("%{v}%")
            });
        }
    }

    (conditions.join("\n                "), params)
}

#[cfg(test)]
#[path = "filters_parse_tests.rs"]
mod parse_tests;

#[cfg(test)]
#[path = "filters_tests.rs"]
mod build_tests;
