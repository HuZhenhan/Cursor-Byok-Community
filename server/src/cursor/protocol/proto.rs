//! Includes generated Cursor protobuf types.
pub mod agent {
    #[allow(clippy::large_enum_variant)]
    pub mod v1 {
        include!(concat!(env!("OUT_DIR"), "/agent.v1.rs"));
    }
}

pub mod aiserver {
    pub mod v1 {
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct BidiRequestId {
            #[prost(string, tag = "1")]
            pub request_id: String,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct BidiAppendRequest {
            #[prost(string, tag = "1")]
            pub data: String,
            #[prost(message, optional, tag = "2")]
            pub request_id: Option<BidiRequestId>,
            #[prost(int64, tag = "3")]
            pub append_seqno: i64,
            #[prost(bytes = "vec", tag = "4")]
            pub data_binary: Vec<u8>,
        }

        /// Commit message generation request. Only the fields the local
        /// generator consumes are decoded; credentials and heavyweight context
        /// fields are intentionally left to prost's unknown-field skipping.
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct WriteGitCommitMessageRequest {
            #[prost(string, repeated, tag = "1")]
            pub diffs: Vec<String>,
            #[prost(string, repeated, tag = "2")]
            pub previous_commit_messages: Vec<String>,
            #[prost(message, optional, tag = "3")]
            pub explicit_context: Option<ExplicitContext>,
        }

        /// Cursor Tab 的 `StreamCpp` 请求。只解码本地补全需要的字段，
        /// 其余字段由 prost 跳过。
        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct StreamCppRequest {
            #[prost(message, optional, tag = "1")]
            pub current_file: Option<CurrentFileInfo>,
            #[prost(string, repeated, tag = "2")]
            pub diff_history: Vec<String>,
            #[prost(string, optional, tag = "3")]
            pub model_name: Option<String>,
            #[prost(message, repeated, tag = "7")]
            pub file_diff_histories: Vec<CppFileDiffHistory>,
            #[prost(message, repeated, tag = "8")]
            pub merged_diff_histories: Vec<CppFileDiffHistory>,
            #[prost(bool, optional, tag = "12")]
            pub immediately_ack: Option<bool>,
            #[prost(message, repeated, tag = "13")]
            pub context_items: Vec<CppContextItem>,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct CurrentFileInfo {
            #[prost(string, tag = "1")]
            pub relative_workspace_path: String,
            #[prost(string, tag = "2")]
            pub contents: String,
            #[prost(message, optional, tag = "3")]
            pub cursor_position: Option<CursorPosition>,
            #[prost(string, tag = "5")]
            pub language_id: String,
            #[prost(message, optional, tag = "6")]
            pub selection: Option<CursorRange>,
            #[prost(int32, tag = "8")]
            pub total_number_of_lines: i32,
            #[prost(int32, tag = "9")]
            pub contents_start_at_line: i32,
            #[prost(string, tag = "19")]
            pub workspace_root_path: String,
            #[prost(string, optional, tag = "20")]
            pub line_ending: Option<String>,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct CursorPosition {
            #[prost(int32, tag = "1")]
            pub line: i32,
            #[prost(int32, tag = "2")]
            pub column: i32,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct CursorRange {
            #[prost(message, optional, tag = "1")]
            pub start_position: Option<CursorPosition>,
            #[prost(message, optional, tag = "2")]
            pub end_position: Option<CursorPosition>,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct CppContextItem {
            #[prost(string, tag = "1")]
            pub contents: String,
            #[prost(string, optional, tag = "2")]
            pub symbol: Option<String>,
            #[prost(string, tag = "3")]
            pub relative_workspace_path: String,
            #[prost(float, tag = "4")]
            pub score: f32,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct CppFileDiffHistory {
            #[prost(string, tag = "1")]
            pub file_name: String,
            #[prost(string, repeated, tag = "2")]
            pub diff_history: Vec<String>,
            #[prost(double, repeated, tag = "3")]
            pub diff_history_timestamps: Vec<f64>,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct LineRange {
            #[prost(int32, tag = "1")]
            pub start_line_number: i32,
            #[prost(int32, tag = "2")]
            pub end_line_number_inclusive: i32,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct StreamCppResponse {
            #[prost(string, tag = "1")]
            pub text: String,
            #[prost(int32, optional, tag = "2")]
            pub suggestion_start_line: Option<i32>,
            #[prost(bool, optional, tag = "4")]
            pub done_stream: Option<bool>,
            #[prost(message, optional, tag = "11")]
            pub range_to_replace: Option<LineRange>,
            #[prost(bool, optional, tag = "13")]
            pub done_edit: Option<bool>,
            #[prost(message, optional, tag = "14")]
            pub model_info: Option<StreamCppResponseModelInfo>,
            #[prost(bool, optional, tag = "15")]
            pub begin_edit: Option<bool>,
            #[prost(bool, optional, tag = "16")]
            pub should_remove_leading_eol: Option<bool>,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct StreamCppResponseModelInfo {
            #[prost(bool, optional, tag = "1")]
            pub is_fused_cursor_prediction_model: Option<bool>,
            #[prost(bool, optional, tag = "2")]
            pub is_multidiff_model: Option<bool>,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct ExplicitContext {
            #[prost(string, tag = "1")]
            pub context: String,
            #[prost(string, optional, tag = "2")]
            pub repo_context: Option<String>,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct WriteGitCommitMessageResponse {
            #[prost(string, tag = "1")]
            pub commit_message: String,
        }

        #[derive(Clone, Copy, PartialEq, ::prost::Message)]
        pub struct BidiAppendResponse {}

        /// `NetworkService/IsConnected` reply. The Cursor extension probes this
        /// ~10s after any slow request starts; a non-OK result is treated as
        /// "network disconnected" and aborts in-flight work (e.g. commit message
        /// generation) even while the model is still streaming, so it always
        /// answers as connected.
        #[derive(Clone, Copy, PartialEq, ::prost::Message)]
        pub struct IsConnectedResponse {}

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct CustomErrorDetails {
            #[prost(string, tag = "1")]
            pub title: String,
            #[prost(string, tag = "2")]
            pub detail: String,
            #[prost(bool, optional, tag = "3")]
            pub allow_command_links_potentially_unsafe_please_only_use_for_handwritten_trusted_markdown:
                Option<bool>,
            #[prost(bool, optional, tag = "4")]
            pub is_retryable: Option<bool>,
            #[prost(bool, optional, tag = "5")]
            pub show_request_id: Option<bool>,
            #[prost(bool, optional, tag = "6")]
            pub should_show_immediate_error: Option<bool>,
        }

        #[derive(Clone, PartialEq, ::prost::Message)]
        pub struct ErrorDetails {
            #[prost(enumeration = "error_details::Error", tag = "1")]
            pub error: i32,
            #[prost(message, optional, tag = "2")]
            pub details: Option<CustomErrorDetails>,
            #[prost(bool, optional, tag = "3")]
            pub is_expected: Option<bool>,
        }

        pub mod error_details {
            #[derive(
                Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration,
            )]
            #[repr(i32)]
            pub enum Error {
                Unspecified = 0,
                CustomMessage = 29,
                ProviderError = 57,
                Internal = 59,
            }
        }
    }
}
