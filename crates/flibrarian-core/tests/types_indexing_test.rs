use flibrarian_core::indexing::types::IndexingMode;

#[test]
fn indexing_mode_from_str_full() {
    assert_eq!(
        IndexingMode::from_str_with_archives("full", None),
        IndexingMode::Full
    );
}

#[test]
fn indexing_mode_from_str_new() {
    assert_eq!(
        IndexingMode::from_str_with_archives("new", None),
        IndexingMode::New
    );
}

#[test]
fn indexing_mode_from_str_search() {
    assert_eq!(
        IndexingMode::from_str_with_archives("search", None),
        IndexingMode::Search
    );
}

#[test]
fn indexing_mode_from_str_pick() {
    let archives = vec!["a.zip".to_string(), "b.zip".to_string()];
    assert_eq!(
        IndexingMode::from_str_with_archives("pick", Some(archives.clone())),
        IndexingMode::Archives(archives)
    );
}

#[test]
fn indexing_mode_from_str_unknown_becomes_archives() {
    assert_eq!(
        IndexingMode::from_str_with_archives("unknown", None),
        IndexingMode::Archives(vec!["unknown".to_string()])
    );
}

#[test]
fn indexing_mode_from_str_pick_empty_archives() {
    assert_eq!(
        IndexingMode::from_str_with_archives("pick", None),
        IndexingMode::Archives(vec![])
    );
}
