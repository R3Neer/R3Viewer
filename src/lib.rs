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
