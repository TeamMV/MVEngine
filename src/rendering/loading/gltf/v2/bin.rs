use mvengine_ui_parsing::json::from_json::FromJsonTrait;
use mvengine_ui_parsing::json::types::JsonElement;
use mvengine_ui_parsing::json::from_json::FromJsonError;
use mvengine_proc_macro::FromJson;

#[derive(FromJson)]
pub struct TestStruct {
    name: String,
    number: i32
}
