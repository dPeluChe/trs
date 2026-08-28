use super::{compress_pull, compress_table};

// Captured from a real `ollama list`, padding and all.
const LIST: &str = "\
NAME                           ID              SIZE      MODIFIED      
gemma4:12b                     4eb23ef187e2    7.6 GB    8 weeks ago      
qwen3-vl:latest                901cae732162    6.1 GB    8 weeks ago      
nemotron-3-ultra:cloud         6d55374b63bb    -         2 months ago     
";

#[test]
fn table_drops_the_id_column_and_the_padding() {
    let out = compress_table(LIST).unwrap();
    assert!(
        out.contains("gemma4:12b  7.6 GB  8 weeks ago"),
        "bad row: {out}"
    );
    // The digest is only useful to `ollama rm`; agents address models by name.
    assert!(!out.contains("4eb23ef187e2"), "ID column survived: {out}");
    assert!(out.contains("3 models"), "missing footer: {out}");
    assert!(out.len() < LIST.len());
}

#[test]
fn table_keeps_a_dash_size_rather_than_dropping_the_row() {
    // Cloud models report no size; the row still matters.
    let out = compress_table(LIST).unwrap();
    assert!(
        out.contains("nemotron-3-ultra:cloud  -  2 months ago"),
        "{out}"
    );
}

#[test]
fn table_declines_a_header_with_no_rows() {
    // `ollama ps` with nothing running is already minimal.
    assert_eq!(
        compress_table("NAME    ID    SIZE    PROCESSOR    UNTIL \n"),
        None
    );
    assert_eq!(compress_table(""), None);
    assert_eq!(compress_table("some other output\n"), None);
}

// Captured from a real `ollama pull`, carriage returns and cursor codes cut.
const PULL: &str =
    "pulling manifest \npulling 667b0c1932bc: 100% ▕████▏ 4.9 GB                         \r\
pulling 948af2743fc7: 100% ▕████▏ 1.5 KB                         \n\
verifying sha256 digest \nwriting manifest \nsuccess";

#[test]
fn pull_folds_progress_into_layers_and_a_verdict() {
    let out = compress_pull(PULL).unwrap();
    assert!(out.contains("pulled 2 layer(s)"), "{out}");
    // The size is two tokens; keeping only the last leaves a bare unit.
    assert!(
        out.contains("667b0c1932bc 4.9 GB"),
        "size lost its number: {out}"
    );
    assert!(out.contains("948af2743fc7 1.5 KB"), "{out}");
    assert!(
        out.trim_end().ends_with("success"),
        "verdict missing: {out}"
    );
    assert!(out.len() < PULL.len());
}

#[test]
fn pull_reports_an_unfinished_download_as_such() {
    // No verdict line: the download did not get to say how it went, and the
    // summary must not imply it succeeded.
    let out = compress_pull("pulling manifest \npulling abc123: 100% ▕██▏ 1 KB\n").unwrap();
    assert!(out.contains("incomplete"), "must not imply success: {out}");
}

#[test]
fn pull_declines_output_that_is_not_a_pull() {
    assert_eq!(compress_pull("NAME    ID\nfoo    bar\n"), None);
    assert_eq!(compress_pull(""), None);
}
