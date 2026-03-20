use buyu_lib::commands::incremental::build_patch_event;
use buyu_lib::domain::incremental::IncrementalPatchOp;
use serde_json::json;

#[test]
fn build_patch_event_captures_upsert_replace_and_delete_shapes() {
    let upsert = build_patch_event(
        "conversation",
        Some("conv_1"),
        "message_version",
        Some("msg_1"),
        IncrementalPatchOp::Upsert,
        json!({ "id": "msg_1", "title": "hello" }),
    );
    assert!(matches!(upsert.op, IncrementalPatchOp::Upsert));
    assert_eq!(upsert.scope_kind, "conversation");
    assert_eq!(upsert.scope_id.as_deref(), Some("conv_1"));
    assert_eq!(upsert.resource_kind, "message_version");
    assert_eq!(upsert.resource_id.as_deref(), Some("msg_1"));
    assert_eq!(upsert.data["title"], "hello");

    let replace = build_patch_event(
        "conversation",
        Some("conv_1"),
        "visible_messages",
        None,
        IncrementalPatchOp::Replace,
        json!([{ "id": "msg_1" }]),
    );
    assert!(matches!(replace.op, IncrementalPatchOp::Replace));
    assert_eq!(replace.resource_kind, "visible_messages");
    assert!(replace.resource_id.is_none());
    assert!(replace.data.is_array());

    let delete = build_patch_event(
        "conversation",
        Some("conv_1"),
        "message_version",
        Some("msg_1"),
        IncrementalPatchOp::Delete,
        serde_json::Value::Null,
    );
    assert!(matches!(delete.op, IncrementalPatchOp::Delete));
    assert_eq!(delete.resource_kind, "message_version");
    assert_eq!(delete.resource_id.as_deref(), Some("msg_1"));
    assert!(delete.data.is_null());
}
