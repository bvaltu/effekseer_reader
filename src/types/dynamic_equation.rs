//! Types for decoded dynamic equation bytecode.

/// A decoded dynamic equation, ready for CPU evaluation or WGSL compilation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DynamicEquation {
    /// Bytecode version (currently always 0 in Effekseer).
    pub version: i32,
    /// When this equation executes.
    pub phase: RunningPhase,
    /// Number of temporary registers used.
    pub register_count: u32,
    /// Which registers hold the 4 output values.
    pub output_registers: [RegRef; 4],
    /// The linear sequence of operations.
    pub operations: Vec<DynOp>,
}

/// Execution phase of a dynamic equation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(i32)]
pub enum RunningPhase {
    /// Executed once per frame at the effect level. Results stored globally.
    Global = 0,
    /// Executed per-particle during parameter application.
    Local = 1,
}

/// A single operation in a dynamic equation.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DynOp {
    /// The operation to perform.
    pub op_type: DynOpType,
    /// Input register references (read before execution).
    pub inputs: Vec<RegRef>,
    /// Output register references (written after execution). Must be temporary registers.
    pub outputs: Vec<RegRef>,
    /// Attribute values (used by `Constant` — stores the f32 bits as i32).
    pub attributes: Vec<i32>,
}

/// Dynamic equation opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(i32)]
pub enum DynOpType {
    /// Load a constant float from attributes[0] (bit-cast from i32).
    Constant = 0,
    /// output = input[0] + input[1]
    Add = 1,
    /// output = input[0] - input[1]
    Sub = 2,
    /// output = input[0] * input[1]
    Mul = 3,
    /// output = input[0] / input[1]
    Div = 4,
    /// output = fmod(input[0], input[1])
    Mod = 5,
    /// output = +input[0] (identity)
    UnaryAdd = 11,
    /// output = -input[0] (negate)
    UnarySub = 12,
    /// output[j] = sin(input[j])
    Sine = 21,
    /// output[j] = cos(input[j])
    Cos = 22,
    /// output = rand() — unseeded random in [0, 1)
    Rand = 31,
    /// output[j] = rand_seed(input[j]) — seeded random
    RandWithSeed = 32,
    /// output = (input[1] >= input[0]) ? 1.0 : 0.0
    Step = 50,
}

/// A reference to a register in the equation VM.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RegRef {
    /// Temporary register (index 0..register_count-1).
    Temp(u32),
    /// External input parameter (index 0..3, from `dynamicInputParameters`).
    External(u32),
    /// Global time register (always index 0 = `updatedFrame / 60.0`).
    Global(u32),
    /// Local particle register (0..3 = parameter values, 4 = living time in seconds).
    Local(u32),
}
