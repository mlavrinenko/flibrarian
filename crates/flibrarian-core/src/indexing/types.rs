use anyhow::{Context, Result};
use duckdb::types::{FromSql, FromSqlError, ToSql, ToSqlOutput};
use fb2::LocalizedText;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use std::sync::LazyLock;

static FB2_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^(\d+)\.fb2$").unwrap());

#[derive(Debug, Serialize)]
pub struct IndexState {
    pub archives_indexed: usize,
    pub archives_pending: usize,
    pub archives_new: usize,
    pub search_index_valid: bool,
    pub total_books: usize,
}

impl IndexState {
    #[must_use]
    pub const fn needs_resume(&self) -> bool {
        self.archives_pending > 0 || self.archives_new > 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IndexingMode {
    Full,
    New,
    Search,
    Archives(Vec<String>),
}

impl IndexingMode {
    #[must_use]
    pub fn from_str_with_archives(mode: &str, archives: Option<Vec<String>>) -> Self {
        match mode {
            "full" => Self::Full,
            "new" => Self::New,
            "search" => Self::Search,
            "pick" => Self::Archives(archives.unwrap_or_default()),
            other => Self::Archives(vec![other.to_string()]),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ArchiveInfo {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum IndexingPhase {
    Counting,
    Parsing,
    Writing,
    BuildingSearchIndex,
    CreatingFtsIndex,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveStatus {
    Indexing,
    Indexed,
}

impl ArchiveStatus {
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Indexing => "indexing",
            Self::Indexed => "indexed",
        }
    }
}

impl FromStr for ArchiveStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "indexing" => Ok(Self::Indexing),
            "indexed" => Ok(Self::Indexed),
            _ => Err(()),
        }
    }
}

impl ToSql for ArchiveStatus {
    fn to_sql(&self) -> duckdb::Result<ToSqlOutput<'_>> {
        Ok(ToSqlOutput::from(self.as_str()))
    }
}

impl FromSql for ArchiveStatus {
    fn column_result(value: duckdb::types::ValueRef) -> duckdb::types::FromSqlResult<Self> {
        value
            .as_str()
            .ok()
            .and_then(|s| Self::from_str(s).ok())
            .ok_or(FromSqlError::InvalidType)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Author {
    pub id: String,
    pub first_name: Option<String>,
    pub middle_name: Option<String>,
    pub last_name: Option<String>,
    pub nickname: Option<String>,
}

impl Display for Author {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<&str> = [
            &self.first_name,
            &self.middle_name,
            &self.last_name,
            &self.nickname,
        ]
        .into_iter()
        .filter_map(|opt| opt.as_deref())
        .filter(|s| !s.is_empty())
        .collect();

        if parts.is_empty() {
            write!(f, "Anonymous")
        } else {
            write!(f, "{}", parts.join(" "))
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Book {
    pub id: u32,
    pub title: String,
    pub genres: Vec<String>,
    pub authors: Vec<Author>,
    pub date: String,
    pub lang: String,
    pub file_size: u64,
    pub sequence: String,
}

impl Book {
    pub fn parse_id(filename: &str) -> Result<u32> {
        FB2_ID_REGEX
            .captures(filename)
            .and_then(|cap| cap.get(1))
            .and_then(|m| m.as_str().parse().ok())
            .context(format!(
                "Failed to extract book ID from FB2 filename: {filename}"
            ))
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct Sequence {
    #[serde(rename = "@name")]
    pub name: Option<String>,
    #[serde(rename = "@number")]
    pub number: Option<String>,
    #[serde(rename = "sequence")]
    pub nested: Option<Vec<Self>>,
}

impl Sequence {
    fn current_to_string(&self) -> String {
        let name = self.name.as_deref().unwrap_or("");
        self.number
            .as_ref()
            .map_or_else(|| name.to_string(), |n| format!("{name} {n}"))
    }

    fn collect_recursive(&self) -> Vec<Self> {
        let mut all = Vec::new();
        all.push(self.clone());
        if let Some(nested_seqs) = &self.nested {
            for nested_seq in nested_seqs {
                all.extend(nested_seq.collect_recursive());
            }
        }
        all
    }
}

impl Display for Sequence {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.collect_recursive()
                .into_iter()
                .map(|seq| seq.current_to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename = "FictionBook")]
pub struct PartialFictionBook {
    pub description: PartialDescription,
}

#[allow(clippy::struct_field_names)]
#[derive(Debug)]
pub struct PartialDescription {
    pub title_info: Vec<PartialTitleInfo>,
    pub publish_info: Option<Vec<PartialPublishInfo>>,
    pub document_info: Option<Vec<PartialDocumentInfo>>,
}

impl<'de> Deserialize<'de> for PartialDescription {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[allow(clippy::struct_field_names)]
        #[derive(Deserialize)]
        struct PartialDescriptionHelper {
            #[serde(rename = "title-info")]
            title_info: Vec<PartialTitleInfo>,
            #[serde(rename = "publish-info", skip_serializing_if = "Option::is_none")]
            publish_info: Option<Vec<PartialPublishInfo>>,
            #[serde(rename = "document-info", skip_serializing_if = "Option::is_none")]
            document_info: Option<Vec<PartialDocumentInfo>>,
        }

        let helper = PartialDescriptionHelper::deserialize(deserializer)?;

        if !helper
            .title_info
            .iter()
            .any(PartialTitleInfo::has_book_title)
        {
            return Err(serde::de::Error::custom(
                "At least one PartialTitleInfo must have a book title",
            ));
        }

        Ok(Self {
            title_info: helper.title_info,
            publish_info: helper.publish_info,
            document_info: helper.document_info,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct PartialTitleInfo {
    #[serde(rename = "genre")]
    pub genres: Option<Vec<String>>,
    #[serde(default, rename = "author")]
    pub authors: Vec<PartialFb2Author>,
    #[serde(rename = "book-title", default)]
    pub book_title: Option<Vec<LocalizedText>>,
    #[serde(rename = "date")]
    pub date: Option<Vec<LocalizedText>>,
    #[serde(default, rename = "sequence")]
    pub sequence: Option<Vec<Sequence>>,
    pub lang: Option<Vec<String>>,
}

impl PartialTitleInfo {
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn has_book_title(&self) -> bool {
        matches!(self.book_title, Some(ref titles) if !titles.is_empty())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct PartialFb2Author {
    #[serde(rename = "first-name")]
    pub first_name: Option<Vec<LocalizedText>>,
    #[serde(rename = "middle-name")]
    pub middle_name: Option<Vec<LocalizedText>>,
    #[serde(rename = "last-name")]
    pub last_name: Option<Vec<LocalizedText>>,
    pub nickname: Option<Vec<LocalizedText>>,
    #[serde(rename = "id")]
    pub id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PartialPublishInfo {
    #[serde(rename = "year")]
    pub year: Option<Vec<LocalizedText>>,
}

#[derive(Debug, Deserialize)]
pub struct PartialDocumentInfo {
    #[serde(rename = "date")]
    pub date: Option<Vec<LocalizedText>>,
}

#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
