// Copyright 2024 Block, Inc. (original code from https://github.com/block/goose)
// Copyright 2025 utapyngo (modifications)
// SPDX-License-Identifier: Apache-2.0

/// Tree-sitter query for extracting Kotlin code elements
pub const ELEMENT_QUERY: &str = r#"
    ; Functions
    (function_declaration name: (identifier) @func)

    ; Classes
    (class_declaration name: (identifier) @class)

    ; Objects (singleton classes)
    (object_declaration name: (identifier) @class)

    ; Imports
    (import) @import
"#;

/// Tree-sitter query for extracting Kotlin function calls
pub const CALL_QUERY: &str = r#"
    ; Simple function calls
    (call_expression
      (identifier) @function.call)

    ; Method calls with navigation (obj.method())
    (call_expression
      (navigation_expression
        (identifier) @method.call))
"#;
