//! Agent subsystem (tool registry, skills, chat loop, harness).

pub mod compress;
pub mod context_budget;
pub mod genai_model;
pub mod harness;
pub mod llama_log;
pub mod llm;
pub mod memory;
pub mod os_shell;
pub mod registry;
pub mod run_registry;
pub mod session_log;
pub mod skills;
pub mod title_summarizer;
pub mod tool_approval;
pub mod trajectory;
pub mod r#loop;
pub mod embedded_engine;