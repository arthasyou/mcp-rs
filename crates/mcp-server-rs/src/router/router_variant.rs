// use std::{future::Future, pin::Pin};

// use mcp_core_rs::protocol::message::{JsonRpcRequest, JsonRpcResponse};
// use mcp_error_rs::Result;

// use crate::router::{
//     impls::{chart::ChartRouter, counter::CounterRouter},
//     traits::Router,
// };

// pub enum RouterVariant {
//     Chart(ChartRouter),
//     Counter(CounterRouter),
// }

// impl Router for RouterVariant {
//     fn name(&self) -> String {
//         match self {
//             RouterVariant::Chart(r) => r.name(),
//             RouterVariant::Counter(r) => r.name(),
//         }
//     }

//     fn instructions(&self) -> String {
//         match self {
//             RouterVariant::Chart(r) => r.instructions(),
//             RouterVariant::Counter(r) => r.instructions(),
//         }
//     }

//     fn capabilities(&self) -> mcp_core_rs::protocol::capabilities::ServerCapabilities {
//         match self {
//             RouterVariant::Chart(r) => r.capabilities(),
//             RouterVariant::Counter(r) => r.capabilities(),
//         }
//     }

//     fn list_tools(&self) -> Vec<mcp_core_rs::Tool> {
//         match self {
//             RouterVariant::Chart(r) => r.list_tools(),
//             RouterVariant::Counter(r) => r.list_tools(),
//         }
//     }

//     fn call_tool(
//         &self,
//         tool_name: &str,
//         arguments: serde_json::Value,
//     ) -> Pin<Box<dyn Future<Output = Result<Vec<mcp_core_rs::content::Content>>> + Send>> {
//         match self {
//             RouterVariant::Chart(r) => r.call_tool(tool_name, arguments),
//             RouterVariant::Counter(r) => r.call_tool(tool_name, arguments),
//         }
//     }

//     fn list_resources(&self) -> Vec<mcp_core_rs::Resource> {
//         match self {
//             RouterVariant::Chart(r) => r.list_resources(),
//             RouterVariant::Counter(r) => r.list_resources(),
//         }
//     }

//     fn read_resource(&self, uri: &str) -> Pin<Box<dyn Future<Output = Result<String>> + Send>> {
//         match self {
//             RouterVariant::Chart(r) => r.read_resource(uri),
//             RouterVariant::Counter(r) => r.read_resource(uri),
//         }
//     }

//     // fn list_prompts(&self) -> Vec<mcp_core_rs::Prompt> {
//     //     match self {
//     //         RouterVariant::Chart(r) => r.list_prompts(),
//     //         RouterVariant::Counter(r) => r.list_prompts(),
//     //     }
//     // }

//     // fn get_prompt(
//     //     &self,
//     //     prompt_name: &str,
//     // ) -> Pin<Box<dyn Future<Output = Result<String>> + Send>> {
//     //     match self {
//     //         RouterVariant::Chart(r) => r.get_prompt(prompt_name),
//     //         RouterVariant::Counter(r) => r.get_prompt(prompt_name),
//     //     }
//     // }

//     // fn create_response(&self, id: Option<u64>) -> JsonRpcResponse {
//     //     match self {
//     //         RouterVariant::Chart(r) => r.create_response(id),
//     //         RouterVariant::Counter(r) => r.create_response(id),
//     //     }
//     // }

//     // fn handle_initialize(
//     //     &self,
//     //     req: JsonRpcRequest,
//     // ) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>> {
//     //     match self {
//     //         RouterVariant::Chart(r) => r.handle_initialize(req),
//     //         RouterVariant::Counter(r) => r.handle_initialize(req),
//     //     }
//     // }

//     // fn handle_tools_list(
//     //     &self,
//     //     req: JsonRpcRequest,
//     // ) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>> {
//     //     match self {
//     //         RouterVariant::Chart(r) => r.handle_tools_list(req),
//     //         RouterVariant::Counter(r) => r.handle_tools_list(req),
//     //     }
//     // }

//     // fn handle_tools_call(
//     //     &self,
//     //     req: JsonRpcRequest,
//     // ) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>> {
//     //     match self {
//     //         RouterVariant::Chart(r) => r.handle_tools_call(req),
//     //         RouterVariant::Counter(r) => r.handle_tools_call(req),
//     //     }
//     // }

//     // fn handle_resources_list(
//     //     &self,
//     //     req: JsonRpcRequest,
//     // ) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>> {
//     //     match self {
//     //         RouterVariant::Chart(r) => r.handle_resources_list(req),
//     //         RouterVariant::Counter(r) => r.handle_resources_list(req),
//     //     }
//     // }

//     // fn handle_resources_read(
//     //     &self,
//     //     req: JsonRpcRequest,
//     // ) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>> {
//     //     match self {
//     //         RouterVariant::Chart(r) => r.handle_resources_read(req),
//     //         RouterVariant::Counter(r) => r.handle_resources_read(req),
//     //     }
//     // }

//     // fn handle_prompts_list(
//     //     &self,
//     //     req: JsonRpcRequest,
//     // ) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>> {
//     //     match self {
//     //         RouterVariant::Chart(r) => r.handle_prompts_list(req),
//     //         RouterVariant::Counter(r) => r.handle_prompts_list(req),
//     //     }
//     // }

//     // fn handle_prompts_get(
//     //     &self,
//     //     req: JsonRpcRequest,
//     // ) -> Pin<Box<dyn Future<Output = Result<JsonRpcResponse>> + Send>> {
//     //     match self {
//     //         RouterVariant::Chart(r) => r.handle_prompts_get(req),
//     //         RouterVariant::Counter(r) => r.handle_prompts_get(req),
//     //     }
//     // }
// }

// impl RouterVariant {
//     pub fn from_service_name(service: &str) -> Self {
//         match service {
//             "chart" => RouterVariant::Chart(ChartRouter::new()),
//             "counter" => RouterVariant::Counter(CounterRouter::new()),
//             _ => RouterVariant::Chart(ChartRouter::new()),
//         }
//     }
// }
