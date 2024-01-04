// SPDX-License-Identifier: Unlicense
use crate::ir;
use anyhow::{anyhow, Result};
use inkwell::{builder::Builder, context::Context, module::Module, targets, values};
use std::{collections::HashMap, path::Path};

#[derive(Debug, Clone)]
struct Value<'a>(Option<values::AnyValueEnum<'a>>);

impl<'a> Value<'a> {
    fn from_int_value(v: values::IntValue<'a>) -> Self {
        Self(Some(values::AnyValueEnum::IntValue(v)))
    }

    fn into_int_value(self) -> Result<values::IntValue<'a>> {
        let v = self
            .0
            .ok_or(anyhow!("expected integer value but actually None"))?;
        if !v.is_int_value() {
            anyhow::bail!("expected integer value but actually {:?}", v)
        }
        Ok(v.into_int_value())
    }
}

pub struct CodeGen<'a> {
    ir_arena: ir::Arena,
    context: &'a Context,
    module: Module<'a>,
    builder: Builder<'a>,
    target_machine: targets::TargetMachine,
}

impl<'a> CodeGen<'a> {
    pub fn new(
        ir_arena: ir::Arena,
        context: &'a Context,
        target_machine: targets::TargetMachine,
        module_name: &str,
    ) -> Self {
        let module = context.create_module(module_name);
        let builder = context.create_builder();
        Self {
            ir_arena,
            context,
            module,
            builder,
            target_machine,
        }
    }

    fn generate_builtins(&self) -> Result<HashMap<&str, values::FunctionValue>> {
        let i64_ty = self.context.i64_type();
        let i8_ptr_ty = self
            .context
            .i8_type()
            .ptr_type(inkwell::AddressSpace::default());
        let void_ty = self.context.void_type();

        let printf = self.module.add_function(
            "printf",
            void_ty.fn_type(&[i8_ptr_ty.into()], true),
            None,
        );

        let print_int = self.module.add_function(
            "print_int",
            void_ty.fn_type(&[i64_ty.into()], false),
            None,
        );
        let print_int_body = self.context.append_basic_block(print_int, "entry");
        self.builder.position_at_end(print_int_body);

        // cf. https://github.com/TheDan64/inkwell/issues/32
        let format_str = unsafe {
            self.builder
                .build_global_string("result: %d\n", "format string")
        };
        let format_str = self.builder.build_cast(
            values::InstructionOpcode::BitCast,
            format_str?.as_pointer_value(),
            i8_ptr_ty,
            "",
        );
        let val_to_print = print_int
            .get_nth_param(0)
            .ok_or(anyhow!("failed to get first param of print_int"))?
            .into_int_value();

        self.builder
            .build_call(printf, &[format_str?.into(), val_to_print.into()], "")?;
        self.builder.build_return(None)?;

        let mut builtins = HashMap::new();
        builtins.insert("print_int", print_int);

        Ok(builtins)
    }

    fn generate_impl(&self, id: ir::Id) -> Result<Value> {
        let kind = &self
            .ir_arena
            .get(id)
            .ok_or(anyhow!("failed to get ir from arena"))?
            .kind;

        match kind {
            &ir::Kind::IntValue(i) => Ok(Value::from_int_value(
                self.context.i64_type().const_int(i as u64, true),
            )),
            ir::Kind::Op(op, args) => {

                let ret = match op {
                    ir::OpKind::IAdd => Value::from_int_value(
                        self.builder.build_int_add(
                            self.generate_impl(args[0])?.into_int_value()?,
                            self.generate_impl(args[1])?.into_int_value()?, 
                        ""
                        )?
                    ),
                    ir::OpKind::ISub => Value::from_int_value(
                        self.builder.build_int_sub(
                            self.generate_impl(args[0])?.into_int_value()?,
                            self.generate_impl(args[1])?.into_int_value()?, 
                        ""
                        )?
                    ),
                    ir::OpKind::IMul => Value::from_int_value(
                        self.builder.build_int_mul(
                            self.generate_impl(args[0])?.into_int_value()?,
                            self.generate_impl(args[1])?.into_int_value()?, 
                        ""
                        )?
                    ),
                    ir::OpKind::IDiv => Value::from_int_value(
                        self.builder.build_int_signed_div(
                            self.generate_impl(args[0])?.into_int_value()?,
                            self.generate_impl(args[1])?.into_int_value()?, 
                        ""
                        )?
                    ),
                };
                Ok(ret)
            }
        }
    }

    pub fn generate(&self, root: ir::Id) -> Result<()> {
        let builtins = self.generate_builtins()?;
        let print_int = builtins
            .get("print_int")
            .ok_or(anyhow!("builtin function not found"))?;

        let ptr_sized_int_ty = self
            .context
            .ptr_sized_int_type(&self.target_machine.get_target_data(), None);

        let main = self
            .module
            .add_function("main", ptr_sized_int_ty.fn_type(&[], false), None);
        let main_body = self.context.append_basic_block(main, "entry");
        self.builder.position_at_end(main_body);

        let val = { self.generate_impl(root)?.into_int_value()? };
        let arg = &[val.into()];

        self.builder.build_call(*print_int, arg, "")?;

        self.builder.build_return(Some(&val))?;

        Ok(())
    }

    pub fn write_to_file(&self, file: &Path) -> Result<()> {
        self.module
            .verify()
            .map_err(|e| anyhow!("module verification failed: {}", e))?;
        self.target_machine
            .write_to_file(&self.module, targets::FileType::Object, file)
            .map_err(|e| anyhow!("failed to write object file: {}", e))
    }
}

pub fn get_host_target_machine() -> Result<targets::TargetMachine> {
    use targets::*;

    Target::initialize_native(&InitializationConfig::default())
        .map_err(|e| anyhow!("failed to initialize native target: {}", e))?;

    let triple = TargetMachine::get_default_triple();
    let target =
        Target::from_triple(&triple).map_err(|e| anyhow!("failed to get target: {}", e))?;

    let cpu = TargetMachine::get_host_cpu_name();
    let features = TargetMachine::get_host_cpu_features();

    let opt_level = inkwell::OptimizationLevel::Default;
    let reloc_mode = RelocMode::Default;
    let code_model = CodeModel::Default;

    target
        .create_target_machine(
            &triple,
            cpu.to_str()?,
            features.to_str()?,
            opt_level,
            reloc_mode,
            code_model,
        )
        .ok_or(anyhow!("failed to get target machine"))
}
