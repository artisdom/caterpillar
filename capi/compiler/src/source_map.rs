use std::collections::BTreeMap;

use capi_runtime::InstructionAddress;

use crate::code::{ExpressionLocation, FunctionLocation};

/// # Mapping of pre-compiled source code to fully compiled instructions
#[derive(
    Clone, Debug, Default, Eq, PartialEq, serde::Deserialize, serde::Serialize,
)]
pub struct SourceMap {
    expression_to_instructions:
        BTreeMap<ExpressionLocation, Vec<InstructionAddress>>,
    instruction_to_expression: BTreeMap<InstructionAddress, ExpressionLocation>,
    function_to_instructions:
        BTreeMap<FunctionLocation, [InstructionAddress; 2]>,
}

impl SourceMap {
    /// # Define a mapping between an expression and a number of instructions
    ///
    /// This function only accepts the location of the expression. To append the
    /// associated instructions, use the returned [`Mapping`].
    pub fn map_fragment_to_instructions(
        &mut self,
        expression: ExpressionLocation,
    ) -> Mapping {
        // Make sure we don't have a previous mapping whose leftovers might
        // corrupt the new one.
        self.expression_to_instructions.remove(&expression);

        Mapping {
            expression,
            source_map: self,
        }
    }

    /// # Define which instructions map to the given function
    pub fn map_function_to_instructions(
        &mut self,
        function: FunctionLocation,
        range: [InstructionAddress; 2],
    ) {
        self.function_to_instructions.insert(function, range);
    }

    /// # Get the location of the expression that the given instruction maps to
    ///
    /// Can return `None`, as there are a few compiler-generated instructions
    /// that call the `main` function.
    pub fn instruction_to_expression(
        &self,
        instruction: &InstructionAddress,
    ) -> Option<&ExpressionLocation> {
        self.instruction_to_expression.get(instruction)
    }

    /// # Get the address of the instruction that the given expression maps to
    ///
    /// Can return a reference to an empty `Vec`, as comments have no mapping to
    /// instructions.
    pub fn fragment_to_instructions(
        &self,
        expression: &ExpressionLocation,
    ) -> &Vec<InstructionAddress> {
        static EMPTY: Vec<InstructionAddress> = Vec::new();

        self.expression_to_instructions
            .get(expression)
            .unwrap_or(&EMPTY)
    }

    /// # Access the function from which this instruction was generated
    ///
    /// Can return `None`, as the instruction that call the `main` function were
    /// not themselves generated by a function.
    pub fn instruction_to_function(
        &self,
        instruction: &InstructionAddress,
    ) -> Option<&FunctionLocation> {
        self.function_to_instructions.iter().find_map(
            |(location, [min, max])| {
                if instruction.index >= min.index
                    && instruction.index <= max.index
                {
                    Some(location)
                } else {
                    None
                }
            },
        )
    }
}

/// # A mapping of an expression to a number of instructions
///
/// Returned by [`SourceMap::define_mapping`].
pub struct Mapping<'r> {
    expression: ExpressionLocation,
    source_map: &'r mut SourceMap,
}

impl Mapping<'_> {
    pub fn append_instruction(&mut self, instruction: InstructionAddress) {
        self.source_map
            .expression_to_instructions
            .entry(self.expression.clone())
            .or_default()
            .push(instruction);
        self.source_map
            .instruction_to_expression
            .insert(instruction, self.expression.clone());
    }
}
