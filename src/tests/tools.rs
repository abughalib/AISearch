#[cfg(test)]
mod tools_test {

    use serde_json::json;

    use crate::tools::tool_creator::{Functions, OpenAITools, Parameters, Properties, Property};

    fn create_openai_test_tool() -> OpenAITools<'static> {
        OpenAITools::default().with_type("function").with_function(
            Functions::default()
                .with_name("test_function_name")
                .with_description("test_function_description")
                .with_parameters(
                    Parameters::default()
                        .with_param_type("test_param_type")
                        .with_properties(
                            Properties::default().add_property(
                                Property::new(
                                    "test_param_name",
                                    "string",
                                    "test_param_description",
                                )
                                .with_required(true),
                            ),
                        ),
                ),
        )
    }

    #[test]
    fn test_tool_create() {
        let test_tool = create_openai_test_tool().build();

        assert!(test_tool.is_ok());

        assert_eq!(
            test_tool.unwrap(),
            json!(
                {
                    "type": "function",
                    "function": {
                        "name": "test_function_name",
                        "description": "test_function_description",
                        "parameters": {
                            "type": "test_param_type",
                            "properties": {
                                "test_param_name": {
                                    "type": "string",
                                    "description": "test_param_description",
                                }
                            },
                            "required": ["test_param_name"]
                        }
                    }
                }
            )
        )
    }

    #[test]
    fn test_tool_create_chat_completion() {
        let test_tool = create_openai_test_tool().as_chat_completion_tool();

        assert!(test_tool.is_ok());
    }
}
