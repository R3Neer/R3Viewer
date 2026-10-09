pub mod cli;
pub mod input;
pub mod output;
pub mod session;
pub mod text;

pub mod config;
pub mod data;
pub mod inspect_text;
pub mod nu_input;
pub mod presentation;
pub mod search;
pub mod viewer_session;

mod language_catalog;
pub mod syntax;
mod syntax_process;
pub mod syntax_selection;
pub mod syntax_worker;

pub mod inline;

pub mod structure_model;
pub mod structure_output;

pub mod worker_process;

pub mod structure;
mod structure_protocol;
pub mod toml_worker;

pub mod json_worker;

pub mod structure_worker;

pub mod yaml_worker;

pub mod yaml_view;

pub mod xml_worker;

pub mod xml_view;

mod text_engine;
mod viewport_admission;

pub mod document_budget;
pub mod document_model;
pub mod document_source;

mod document_obsidian;

mod document_validate;

mod document_utf8;

mod document_parser;

mod document_protocol;

pub mod document_worker;

pub mod document_controller;

pub mod document_text;
mod text_projection;

pub mod document_consumer_budget;

pub mod document_flow;

pub mod document_consumer_profile;
pub mod document_navigation;

mod preparation_owner;
mod source_preparation;

mod document_code;
mod document_code_cache;

#[allow(dead_code)] // Source fallback will be connected by the D3 inline emitter.
mod document_inline_source;

mod document_layout;
mod document_view;

#[allow(dead_code)] // Awaiting the independently reviewed real Markdown caller.
mod document_inline;

mod document_roles;

pub mod document_output;
pub mod document_selection;

mod document_renderer;
mod document_terminal;

#[cfg(test)]
#[path = "document_terminal_storage_tests.rs"]
mod document_terminal_storage_tests;

#[allow(dead_code)] // Independent b3 core awaiting combined adoption.
mod document_search;
