//! Centralized strongly-typed error hierarchy for macicon.

use std::fmt;
use std::io;

/// Central error type for the macicon library and CLI.
#[derive(Debug)]
pub enum MacIconError {
    /// Errors originating from domain validation and invariants.
    Domain(DomainError),
    /// Errors originating from external infrastructure, I/O, or macOS subsystem utilities.
    Infra(InfraError),
    /// Errors originating from CLI parsing, missing arguments, or invalid user inputs.
    Cli(String),
    /// Icon was not found online or in known vectors.
    NotFound { query: String, details: String },
}

impl fmt::Display for MacIconError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MacIconError::Domain(err) => write!(f, "{}", err),
            MacIconError::Infra(err) => write!(f, "{}", err),
            MacIconError::Cli(msg) => write!(f, "{}", msg),
            MacIconError::NotFound { query, details } => {
                write!(
                    f,
                    "Could not find icon '{}' on Simple Icons or Dashboard Icons.\n\n{}",
                    query, details
                )
            }
        }
    }
}

impl std::error::Error for MacIconError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MacIconError::Domain(err) => Some(err),
            MacIconError::Infra(err) => Some(err),
            MacIconError::Cli(_) => None,
            MacIconError::NotFound { .. } => None,
        }
    }
}

impl From<DomainError> for MacIconError {
    fn from(err: DomainError) -> Self {
        MacIconError::Domain(err)
    }
}

impl From<InfraError> for MacIconError {
    fn from(err: InfraError) -> Self {
        MacIconError::Infra(err)
    }
}

impl From<io::Error> for MacIconError {
    fn from(err: io::Error) -> Self {
        MacIconError::Infra(InfraError::Io(err))
    }
}

/// Errors occurring within the domain layer (pure business rules & invariants).
#[derive(Debug, PartialEq, Eq)]
pub enum DomainError {
    /// An invalid hex color was encountered.
    InvalidHexColor(String),
    /// An invalid geometry or scale dimension was specified.
    InvalidGeometry(String),
    /// A requested theme was not found or is invalid.
    InvalidTheme(String),
    /// An invalid icon source or property was specified.
    InvalidIcon(String),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DomainError::InvalidHexColor(c) => write!(f, "Invalid hex color: '{}'", c),
            DomainError::InvalidGeometry(g) => write!(f, "Invalid geometry: {}", g),
            DomainError::InvalidTheme(t) => write!(f, "Invalid theme: '{}'", t),
            DomainError::InvalidIcon(i) => write!(f, "Invalid icon: {}", i),
        }
    }
}

impl std::error::Error for DomainError {}

/// Errors occurring within the infrastructure layer (macOS tools, network, filesystem, rendering).
#[derive(Debug)]
pub enum InfraError {
    /// File system or I/O failure.
    Io(io::Error),
    /// Network request failure.
    Network(String),
    /// SVG parsing error.
    SvgParse(String),
    /// Execution failure of an external macOS tool (e.g., sips, iconutil, qlmanage, swift).
    ToolExecution { tool: String, message: String },
    /// AppleScript / osascript execution failure.
    AppleScriptFailed(String),
    /// Target application bundle was not found.
    AppBundleNotFound(String),
    /// Target icon file was not found.
    IconFileNotFound(String),
    /// JSON serialization/deserialization failure.
    Json(serde_json::Error),
}

impl fmt::Display for InfraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InfraError::Io(err) => write!(f, "I/O error: {}", err),
            InfraError::Network(msg) => write!(f, "Network error: {}", msg),
            InfraError::SvgParse(msg) => write!(f, "SVG parse error: {}", msg),
            InfraError::ToolExecution { tool, message } => {
                write!(f, "Tool '{}' failed: {}", tool, message)
            }
            InfraError::AppleScriptFailed(msg) => write!(f, "AppleScript error: {}", msg),
            InfraError::AppBundleNotFound(path) => {
                write!(f, "Target app '{}' does not exist.", path)
            }
            InfraError::IconFileNotFound(path) => {
                write!(f, "Icon file '{}' does not exist.", path)
            }
            InfraError::Json(err) => write!(f, "JSON error: {}", err),
        }
    }
}

impl std::error::Error for InfraError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            InfraError::Io(err) => Some(err),
            InfraError::Json(err) => Some(err),
            _ => None,
        }
    }
}

impl From<io::Error> for InfraError {
    fn from(err: io::Error) -> Self {
        InfraError::Io(err)
    }
}

impl From<serde_json::Error> for InfraError {
    fn from(err: serde_json::Error) -> Self {
        InfraError::Json(err)
    }
}
