use super::compress_gh_api;
use serde_json::Value;

fn parsed(s: &str) -> Value {
    serde_json::from_str(s).expect("output must stay valid JSON")
}

#[test]
fn drops_link_boilerplate_but_keeps_html_url() {
    let raw = r#"{"number":41,"title":"Fix it","url":"https://api.github.com/repos/o/r/pulls/41","html_url":"https://github.com/o/r/pull/41","node_id":"PR_kwDO","comments_url":"https://api.github.com/x","_links":{"self":{"href":"https://api.github.com/y"}}}"#;
    let out = compress_gh_api(raw).unwrap();
    let v = parsed(&out);
    assert_eq!(v["number"], 41);
    assert_eq!(v["title"], "Fix it");
    // The one link anyone follows survives.
    assert_eq!(v["html_url"], "https://github.com/o/r/pull/41");
    for gone in ["url", "node_id", "comments_url", "_links"] {
        assert!(
            v.get(gone).is_none(),
            "{gone} should have been dropped: {out}"
        );
    }
}

#[test]
fn prunes_nested_objects_and_arrays() {
    let raw = r#"{"head":{"ref":"main","node_id":"X","repo":{"name":"r","forks_url":"u"}},"parents":[{"sha":"a","url":"u","html_url":"h"},{"sha":"b","url":"u","html_url":"h"}]}"#;
    let v = parsed(&compress_gh_api(raw).unwrap());
    assert_eq!(v["head"]["ref"], "main");
    assert!(v["head"].get("node_id").is_none(), "nested key not pruned");
    assert!(
        v["head"]["repo"].get("forks_url").is_none(),
        "deeply nested key not pruned"
    );
    assert_eq!(v["parents"][0]["sha"], "a");
    assert!(
        v["parents"][1].get("url").is_none(),
        "array element not pruned"
    );
    assert_eq!(v["parents"][1]["html_url"], "h");
}

#[test]
fn drops_the_pgp_blob_but_keeps_the_verdict() {
    // `payload` restates fields already present as structured keys, and
    // `signature` is armor. `verified`/`reason` are what anyone reads.
    let raw = r#"{"sha":"abc","commit":{"message":"fix","verification":{"verified":true,"reason":"valid","signature":"-----BEGIN PGP-----AAAA","payload":"tree 1234\nauthor x"}}}"#;
    let v = parsed(&compress_gh_api(raw).unwrap());
    let ver = &v["commit"]["verification"];
    assert_eq!(ver["verified"], true);
    assert_eq!(ver["reason"], "valid");
    assert!(ver.get("signature").is_none(), "signature kept");
    assert!(ver.get("payload").is_none(), "payload kept");
    // A `payload` outside a verification block is real content and stays.
    let other = r#"{"payload":"keep me","signature":"keep me too","padding":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}"#;
    assert_eq!(
        compress_gh_api(other),
        None,
        "nothing to drop, so passthrough"
    );
}

#[test]
fn declines_non_json_bodies() {
    // An error page, raw file content, or an empty `--method DELETE` body.
    assert_eq!(compress_gh_api("gh: Not Found (HTTP 404)"), None);
    assert_eq!(compress_gh_api(""), None);
    assert_eq!(compress_gh_api("   \n"), None);
}

#[test]
fn never_returns_more_than_it_received() {
    // A response with no boilerplate has nothing to gain, and re-encoding
    // could add bytes (whitespace normalization going the wrong way).
    let lean = r#"{"a":1,"b":2}"#;
    assert_eq!(compress_gh_api(lean), None);
    let pretty = "{\n  \"a\": 1,\n  \"b\": 2\n}";
    // Pretty-printed input is safe to compact: same value, fewer bytes.
    let out = compress_gh_api(pretty).unwrap();
    assert_eq!(parsed(&out)["a"], 1);
    assert!(out.len() < pretty.len());
}

#[test]
fn output_is_a_single_line_of_valid_json() {
    let raw = r#"{"title":"a","body":"line one\nline two","url":"u","node_id":"n","padding":"xxxxxxxxxxxxxxxxxxxx"}"#;
    let out = compress_gh_api(raw).unwrap();
    assert_eq!(out.lines().count(), 1, "must not wrap: {out}");
    // A newline inside a string value stays escaped, not literal.
    assert_eq!(parsed(&out)["body"], "line one\nline two");
}
