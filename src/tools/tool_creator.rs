use anyhow::Result;
use async_openai::types::{ChatCompletionTool, ChatCompletionToolArgs, FunctionObjectArgs};
use serde::{ser::SerializeMap, Serialize};
use serde_json::{json, Value};

#[derive(Debug, Serialize)]
pub struct OpenAITools<'a> {
    #[serde(rename(serialize = "type"))]
    tool_type: &'a str,
    function: Functions<'a>,
}

impl<'a> OpenAITools<'a> {
    pub fn default() -> Self {
        Self {
            tool_type: "function",
            function: Functions::default(),
        }
    }
    pub fn with_type(mut self, tool_type: &'a str) -> Self {
        self.tool_type = tool_type;
        self
    }
    pub fn with_function(mut self, function: Functions<'a>) -> Self {
        self.function = function;
        self
    }

    pub fn build(self) -> Result<Value> {
        let json = serde_json::to_value(self)?;
        Ok(json)
    }

    pub fn as_chat_completion_tool(self) -> Result<ChatCompletionTool> {
        Ok(ChatCompletionToolArgs::default()
            .function(
                FunctionObjectArgs::default()
                    .name(self.function.name)
                    .description(self.function.description)
                    .parameters::<Value>(serde_json::to_value(self.function.parameters)?)
                    .build()?,
            )
            .build()?)
    }
}

#[derive(Debug, Serialize)]
pub struct Functions<'a> {
    name: &'a str,
    description: &'a str,
    parameters: Parameters<'a>,
}

impl<'a> Functions<'a> {
    pub fn default() -> Self {
        Self {
            name: "",
            description: "",
            parameters: Parameters::default(),
        }
    }
    pub fn with_name(mut self, name: &'a str) -> Self {
        self.name = name;
        self
    }
    pub fn with_description(mut self, description: &'a str) -> Self {
        self.description = description;
        self
    }
    pub fn with_parameters(mut self, parameters: Parameters<'a>) -> Self {
        self.parameters = parameters;
        self
    }
}

#[derive(Debug, Serialize)]
pub struct Parameters<'a> {
    #[serde(rename(serialize = "type"))]
    param_type: &'a str,
    properties: Properties<'a>,
    required: Vec<&'a str>,
}

impl<'a> Parameters<'a> {
    pub fn default() -> Self {
        Self {
            param_type: "object",
            properties: Properties::default(),
            required: Vec::new(),
        }
    }

    pub fn with_param_type(mut self, param_type: &'a str) -> Self {
        self.param_type = param_type;
        self
    }
    pub fn with_properties(mut self, properties: Properties<'a>) -> Self {
        let mut required: Vec<&str> = Vec::new();

        for prop in &properties.props {
            if prop.required {
                required.push(prop.variable_name);
            }
        }

        self.properties = properties;
        self.required = required;

        self
    }
}

#[derive(Debug)]
pub struct Properties<'a> {
    props: Vec<Property<'a>>,
}

impl<'a> Properties<'a> {
    pub fn default() -> Self {
        Self { props: Vec::new() }
    }
    pub fn add_property(mut self, prop: Property<'a>) -> Self {
        self.props.push(prop);
        self
    }
}

impl Serialize for Properties<'_> {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.props.len()))?;

        for prop in self.props.iter() {
            let mut prop_map = serde_json::Map::new();

            prop_map.insert("type".to_owned(), json!(prop.variable_type));

            if !prop.variable_description.is_empty() {
                prop_map.insert("description".to_owned(), json!(prop.variable_description));
            }

            if !prop.enum_values.is_empty() {
                prop_map.insert("enum".to_owned(), json!(prop.enum_values));
            }

            map.serialize_entry(&prop.variable_name, &json!(prop_map))?;
        }

        map.end()
    }
}

#[derive(Debug)]
pub struct Property<'a> {
    variable_name: &'a str,
    variable_type: &'a str,
    variable_description: &'a str,
    enum_values: Vec<&'a str>,
    required: bool,
}

impl<'a> Property<'a> {
    pub fn new(
        variable_name: &'a str,
        variable_type: &'a str,
        variable_description: &'a str,
    ) -> Self {
        Self {
            variable_name,
            variable_type,
            variable_description,
            enum_values: Vec::new(),
            required: false,
        }
    }
    pub fn with_enum_values(mut self, enum_values: Vec<&'a str>) -> Self {
        self.enum_values = enum_values;
        self
    }
    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}
