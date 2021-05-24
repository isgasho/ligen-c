use std::{fs::File, io::Write, path::Path};

use ligen_core::{
    ir::{
        Attribute, Attributes, Implementation,
        ImplementationItem::{Constant, Method},
    },
    proc_macro::Context,
    utils::Logger,
};

use crate::ast::Type;
use std::fs::create_dir_all;

#[derive(Debug, Copy, Clone)]
/// Logger struct used for Display in the ligen crates
pub struct BindingGenerator {
    sized_integer: bool,
}

impl Default for BindingGenerator {
    fn default() -> Self {
        Self {
            sized_integer: false,
        }
    }
}

impl BindingGenerator {
    /// function to create a new BindingGenerator
    pub fn new(attr: &Attributes) -> Self {
        let mut sized_integer = false;
        if attr.attributes.iter().any(|attribute| {
            if let Attribute::Named(ident, lit) = attribute {
                if (ident.name.as_str(), lit.to_string().as_str()) == ("integer", "sized") {
                    true
                } else {
                    false
                }
            } else {
                false
            }
        }) {
            sized_integer = true;
        }

        Self { sized_integer }
    }

    /// generate function for the BindingGenerator
    pub fn generate(&self, context: &Context, implementation: &Implementation) {
        let mut statements = vec![
            String::from("#pragma once"),
            String::from("#include <stdint.h>"),
            String::from("#ifdef __cplusplus"),
            String::from("extern \"C\" {"),
            String::from("#endif"),
            String::from(format!("struct {} {}", implementation.self_.name, "{")),
            String::from("void* self;"),
            String::from("}"),
        ];

        for item in &implementation.items {
            match item {
                Constant(_) => Logger::log("Const extern not supported."),
                Method(method) => {
                    match &method.output {
                        None => (),
                        Some(typ) => match typ {
                            ligen_core::ir::Type::Atomic(_)
                            | ligen_core::ir::Type::Reference(_) => (),
                            ligen_core::ir::Type::Compound(_) => continue,
                        },
                    };
                    let name =
                        format!("{}_{}", &implementation.self_.name, &method.identifier.name);

                    let mut inner_types: Vec<String> = vec![];
                    method.input.iter().for_each(|parameter| {
                        let ident = &parameter.identifier.name;
                        let typ = &parameter.type_;
                        inner_types.push(format!(
                            "{} {}",
                            format!("{}", Type::from(typ.clone())),
                            ident
                        ));
                    });

                    //TODO: Distinguish sized types

                    statements.push(String::from(format!(
                        "{} {}({});",
                        if let Some(typ) = &method.output {
                            format!("{}", Type::from(typ.clone()))
                        } else {
                            String::from("void")
                        },
                        &name,
                        inner_types.join(", ")
                    )));
                }
            }
        }

        statements.extend_from_slice(&[
            String::from("#ifdef __cplusplus"),
            String::from("}"),
            String::from("#endif"),
        ]);

        let header_path = context
            .arguments
            .target_dir
            .join("ligen")
            .join(&context.arguments.crate_name)
            .join("include")
            .join(format!("{}.h", implementation.self_.name));

        let mut file = File::create(&header_path)
            .expect(&format!("Failed to create file {}.", header_path.display()));
        file.write_all(statements.join("\n").as_bytes())
            .expect("Failed to write file");
    }
}

// #[cfg(test)]
// mod test {
//     use std::{convert::TryFrom, path::PathBuf};

//     use ligen_core::{
//         ir::{Attribute, Attributes, Identifier, Implementation, Literal},
//         proc_macro::{Context, SourceFile},
//     };
//     use quote::quote;

//     use super::BindingGenerator;

//     #[test]
//     fn bindings() {
//         let generator = BindingGenerator::new(&Attributes {
//             attributes: vec![Attribute::Named(
//                 Identifier::new("integer"),
//                 Literal::String(String::from("sized")),
//             )],
//         });

//         assert_eq!(
//             generator.generate(
//                 Implementation::try_from(quote! {impl Test {
//                     pub fn sum(x: i32, y: i32) -> i32 {
//                         x + y
//                     }
//                 }})
//                 .expect("Failed to parse implementation"),
//                 Context {
//                     source_file: SourceFile {
//                         is_real: true,
//                         path: PathBuf::from("src/adder.rs")
//                     }
//                 }
//             ),
//             [
//                 "#pragma once",
//                 "#include <stdint.h>",
//                 "#ifdef __cplusplus",
//                 "extern \"C\" {",
//                 "#endif",
//                 "struct Test {",
//                 "void* self;",
//                 "}",
//                 "int Test_sum(int x, int y);",
//                 "#ifdef __cplusplus",
//                 "}",
//                 "#endif",
//             ]
//         );
//     }
// }
