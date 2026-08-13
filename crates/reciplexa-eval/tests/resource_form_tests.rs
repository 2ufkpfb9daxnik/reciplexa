//! Language-only `(resource …)` evals to an observable package-resource record.

use reciplexa_eval::{eval_source, eval_source_with_host, MemoryFsHost, RuntimeValue, UnitHost};

fn field<'a>(v: &'a RuntimeValue, name: &str) -> Option<&'a RuntimeValue> {
    let RuntimeValue::Record(fields) = v else {
        return None;
    };
    fields.iter().find(|(k, _)| k == name).map(|(_, v)| v)
}

fn as_str(v: &RuntimeValue) -> Option<&str> {
    match v {
        RuntimeValue::String(s) => Some(s.as_str()),
        _ => None,
    }
}

#[test]
fn eval_resource_with_unit_host_is_package_resource_record() {
    let v = eval_source(r#"(val main (resource "fonts/main.otf"))"#).unwrap();
    assert_eq!(as_str(field(&v, "tag").unwrap()), Some("package-resource"));
    assert_eq!(as_str(field(&v, "path").unwrap()), Some("fonts/main.otf"));
    assert_eq!(
        as_str(field(&v, "note").unwrap()),
        Some("resolve at package load")
    );
}

#[test]
fn eval_resource_with_memory_fs_host_still_deferred() {
    let mut host = MemoryFsHost::default();
    host.files
        .insert("fonts/main.otf".into(), "unused".into());
    let v = eval_source_with_host(r#"(val main (resource "fonts/main.otf"))"#, &mut host).unwrap();
    // Pure identity — no FS read; package root unavailable → deferred record.
    assert_eq!(as_str(field(&v, "tag").unwrap()), Some("package-resource"));
    assert_eq!(as_str(field(&v, "path").unwrap()), Some("fonts/main.otf"));
    let _ = UnitHost;
}

#[test]
fn field_access_on_resource_handle() {
    let v = eval_source(
        r#"(val main (field (resource "styles/default.css") path))"#,
    )
    .unwrap();
    assert_eq!(v, RuntimeValue::String("styles/default.css".into()));
}
