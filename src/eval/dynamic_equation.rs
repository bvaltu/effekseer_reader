//! Dynamic equation bytecode decoder and CPU evaluator.
//!
//! Decodes the opaque `Vec<u8>` bytecode blobs stored in [`Effect::dynamic_equations`]
//! into structured [`DynamicEquation`] values, and provides a CPU evaluator for
//! global-phase equations.
//!
//! # Bytecode format (from C++ `InternalScript`)
//!
//! ## Header (all little-endian i32):
//! | Offset | Field              |
//! |--------|--------------------|
//! | 0      | version            |
//! | 4      | running_phase      |
//! | 8      | register_count     |
//! | 12     | operator_count     |
//! | 16     | output_register[0] |
//! | 20     | output_register[1] |
//! | 24     | output_register[2] |
//! | 28     | output_register[3] |
//!
//! ## Operator stream (repeats `operator_count` times):
//! | Field           | Type  |
//! |-----------------|-------|
//! | op_type         | i32   |
//! | input_count     | i32   |
//! | output_count    | i32   |
//! | attribute_count | i32   |
//! | inputs[..]      | i32[] |
//! | outputs[..]     | i32[] |
//! | attributes[..]  | i32[] |

use crate::types::dynamic_equation::{DynOp, DynOpType, DynamicEquation, RegRef, RunningPhase};

/// Errors that can occur when decoding dynamic equation bytecode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecodeError {
    /// The bytecode blob is too short to contain the header.
    HeaderTooShort {
        /// Actual length of the blob.
        len: usize,
    },
    /// An invalid running phase value was encountered.
    InvalidPhase {
        /// The raw i32 value.
        value: i32,
    },
    /// An invalid opcode was encountered.
    InvalidOpcode {
        /// The raw i32 value.
        value: i32,
        /// Operator index (0-based).
        op_index: usize,
    },
    /// An invalid register reference was encountered.
    InvalidRegister {
        /// The raw i32 register index.
        value: i32,
        /// Operator index (0-based), or `None` for output registers.
        op_index: Option<usize>,
    },
    /// The operator stream ended prematurely.
    UnexpectedEndOfOperators {
        /// Operator index being read when data ran out.
        op_index: usize,
    },
    /// The operator stream has trailing bytes after all operators were read.
    TrailingBytes {
        /// Number of unconsumed bytes.
        remaining: usize,
    },
    /// A negative count was encountered.
    NegativeCount {
        /// Field name.
        field: &'static str,
        /// The raw value.
        value: i32,
    },
    /// An output register in an operator refers to a non-temporary register.
    OutputNotTemporary {
        /// The raw register index.
        value: i32,
        /// Operator index (0-based).
        op_index: usize,
    },
}

impl core::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::HeaderTooShort { len } => {
                write!(f, "equation bytecode too short for header: {len} bytes")
            }
            Self::InvalidPhase { value } => {
                write!(f, "invalid running phase: {value}")
            }
            Self::InvalidOpcode { value, op_index } => {
                write!(f, "invalid opcode {value} at operator {op_index}")
            }
            Self::InvalidRegister { value, op_index } => match op_index {
                Some(i) => write!(f, "invalid register {value:#x} at operator {i}"),
                None => write!(f, "invalid output register {value:#x}"),
            },
            Self::UnexpectedEndOfOperators { op_index } => {
                write!(f, "unexpected end of operator stream at operator {op_index}")
            }
            Self::TrailingBytes { remaining } => {
                write!(f, "{remaining} trailing bytes in operator stream")
            }
            Self::NegativeCount { field, value } => {
                write!(f, "negative {field} count: {value}")
            }
            Self::OutputNotTemporary { value, op_index } => {
                write!(
                    f,
                    "operator {op_index} output register {value:#x} is not a temporary"
                )
            }
        }
    }
}

impl std::error::Error for DecodeError {}

// --- Register address constants (matching C++ InternalScript) ---
const EXTERNAL_BASE: i32 = 0x1000;
const GLOBAL_BASE: i32 = 0x1000 + 0x100;
const LOCAL_BASE: i32 = 0x1000 + 0x200;

const HEADER_SIZE: usize = 8 * 4; // 8 i32 fields

/// Decode a register index from the raw i32 value.
fn decode_reg_ref(value: i32, register_count: u32) -> Option<RegRef> {
    if value < 0 {
        return None;
    }
    let v = value;
    if v < register_count as i32 {
        return Some(RegRef::Temp(v as u32));
    }
    if (EXTERNAL_BASE..=EXTERNAL_BASE + 3).contains(&v) {
        return Some(RegRef::External((v - EXTERNAL_BASE) as u32));
    }
    if v == GLOBAL_BASE {
        return Some(RegRef::Global(0));
    }
    if (LOCAL_BASE..=LOCAL_BASE + 4).contains(&v) {
        return Some(RegRef::Local((v - LOCAL_BASE) as u32));
    }
    None
}

/// Decode a dynamic equation opcode from its raw i32 value.
fn decode_op_type(value: i32) -> Option<DynOpType> {
    match value {
        0 => Some(DynOpType::Constant),
        1 => Some(DynOpType::Add),
        2 => Some(DynOpType::Sub),
        3 => Some(DynOpType::Mul),
        4 => Some(DynOpType::Div),
        5 => Some(DynOpType::Mod),
        11 => Some(DynOpType::UnaryAdd),
        12 => Some(DynOpType::UnarySub),
        21 => Some(DynOpType::Sine),
        22 => Some(DynOpType::Cos),
        31 => Some(DynOpType::Rand),
        32 => Some(DynOpType::RandWithSeed),
        50 => Some(DynOpType::Step),
        _ => None,
    }
}

/// Read a little-endian i32 from a byte slice at `offset`.
///
/// Returns `None` if there aren't enough bytes.
fn read_i32(data: &[u8], offset: usize) -> Option<i32> {
    let bytes: [u8; 4] = data.get(offset..offset + 4)?.try_into().ok()?;
    Some(i32::from_le_bytes(bytes))
}

/// Decode a dynamic equation bytecode blob into a [`DynamicEquation`].
///
/// This validates the bytecode structure, register references, and opcodes.
/// Returns an error if the bytecode is malformed.
pub fn decode(data: &[u8]) -> Result<DynamicEquation, DecodeError> {
    if data.len() < HEADER_SIZE {
        return Err(DecodeError::HeaderTooShort { len: data.len() });
    }

    let version = read_i32(data, 0).unwrap();
    let phase_raw = read_i32(data, 4).unwrap();
    let register_count_raw = read_i32(data, 8).unwrap();
    let operator_count_raw = read_i32(data, 12).unwrap();

    let phase = match phase_raw {
        0 => RunningPhase::Global,
        1 => RunningPhase::Local,
        _ => return Err(DecodeError::InvalidPhase { value: phase_raw }),
    };

    if register_count_raw < 0 {
        return Err(DecodeError::NegativeCount {
            field: "register_count",
            value: register_count_raw,
        });
    }
    let register_count = register_count_raw as u32;

    if operator_count_raw < 0 {
        return Err(DecodeError::NegativeCount {
            field: "operator_count",
            value: operator_count_raw,
        });
    }
    let operator_count = operator_count_raw as u32;

    // Decode output registers
    let mut output_registers = [RegRef::Temp(0); 4];
    for i in 0..4 {
        let raw = read_i32(data, 16 + i * 4).unwrap();
        output_registers[i] = decode_reg_ref(raw, register_count).ok_or(
            DecodeError::InvalidRegister {
                value: raw,
                op_index: None,
            },
        )?;
    }

    // Decode operator stream
    let op_data = &data[HEADER_SIZE..];
    let mut offset = 0usize;
    let mut operations = Vec::with_capacity(operator_count as usize);

    for op_idx in 0..operator_count as usize {
        // Read operator header (4 i32s = 16 bytes)
        if offset + 16 > op_data.len() {
            return Err(DecodeError::UnexpectedEndOfOperators { op_index: op_idx });
        }

        let type_raw = read_i32(op_data, offset).unwrap();
        let input_count_raw = read_i32(op_data, offset + 4).unwrap();
        let output_count_raw = read_i32(op_data, offset + 8).unwrap();
        let attr_count_raw = read_i32(op_data, offset + 12).unwrap();
        offset += 16;

        let op_type = decode_op_type(type_raw).ok_or(DecodeError::InvalidOpcode {
            value: type_raw,
            op_index: op_idx,
        })?;

        if input_count_raw < 0 {
            return Err(DecodeError::NegativeCount {
                field: "input_count",
                value: input_count_raw,
            });
        }
        if output_count_raw < 0 {
            return Err(DecodeError::NegativeCount {
                field: "output_count",
                value: output_count_raw,
            });
        }
        if attr_count_raw < 0 {
            return Err(DecodeError::NegativeCount {
                field: "attribute_count",
                value: attr_count_raw,
            });
        }

        let input_count = input_count_raw as usize;
        let output_count = output_count_raw as usize;
        let attr_count = attr_count_raw as usize;

        let total_payload = (input_count + output_count + attr_count) * 4;
        if offset + total_payload > op_data.len() {
            return Err(DecodeError::UnexpectedEndOfOperators { op_index: op_idx });
        }

        // Read inputs (can reference any valid register)
        let mut inputs = Vec::with_capacity(input_count);
        for _ in 0..input_count {
            let raw = read_i32(op_data, offset).unwrap();
            offset += 4;
            let reg = decode_reg_ref(raw, register_count).ok_or(DecodeError::InvalidRegister {
                value: raw,
                op_index: Some(op_idx),
            })?;
            inputs.push(reg);
        }

        // Read outputs (must be temporary registers)
        let mut outputs = Vec::with_capacity(output_count);
        for _ in 0..output_count {
            let raw = read_i32(op_data, offset).unwrap();
            offset += 4;
            if raw < 0 || raw >= register_count as i32 {
                return Err(DecodeError::OutputNotTemporary {
                    value: raw,
                    op_index: op_idx,
                });
            }
            outputs.push(RegRef::Temp(raw as u32));
        }

        // Read attributes (raw i32 values, used for constants etc.)
        let mut attributes = Vec::with_capacity(attr_count);
        for _ in 0..attr_count {
            let raw = read_i32(op_data, offset).unwrap();
            offset += 4;
            attributes.push(raw);
        }

        operations.push(DynOp {
            op_type,
            inputs,
            outputs,
            attributes,
        });
    }

    if offset != op_data.len() {
        return Err(DecodeError::TrailingBytes {
            remaining: op_data.len() - offset,
        });
    }

    Ok(DynamicEquation {
        version,
        phase,
        register_count,
        output_registers,
        operations,
    })
}

/// Inputs for CPU evaluation of a dynamic equation.
#[derive(Debug, Clone)]
pub struct EvalInputs {
    /// External dynamic input parameters (4 floats, from effect instance).
    pub externals: [f32; 4],
    /// Global time in seconds (`updated_frame / 60.0`).
    pub global_time: f32,
    /// Local particle parameters (4 parameter values + living time in seconds).
    /// Only used for `RunningPhase::Local` equations.
    pub locals: [f32; 5],
}

impl Default for EvalInputs {
    fn default() -> Self {
        Self {
            externals: [0.0; 4],
            global_time: 0.0,
            locals: [0.0; 5],
        }
    }
}

/// A random number generator callback for equation evaluation.
pub trait RandProvider {
    /// Generate a random float in [0, 1).
    fn rand(&mut self) -> f32;
    /// Generate a seeded random float in [0, 1).
    fn rand_with_seed(&mut self, seed: f32) -> f32;
}

/// A simple xorshift-based random provider for testing and default use.
#[derive(Debug, Clone)]
pub struct SimpleRand {
    state: u32,
}

impl SimpleRand {
    /// Create a new random provider with the given seed.
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }
}

impl RandProvider for SimpleRand {
    fn rand(&mut self) -> f32 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 17;
        self.state ^= self.state << 5;
        (self.state as f32) / (u32::MAX as f32)
    }

    fn rand_with_seed(&mut self, seed: f32) -> f32 {
        let seed_bits = seed.to_bits();
        let mut s = if seed_bits == 0 { 1 } else { seed_bits };
        s ^= s << 13;
        s ^= s >> 17;
        s ^= s << 5;
        (s as f32) / (u32::MAX as f32)
    }
}

/// Evaluate a decoded dynamic equation on the CPU.
///
/// Returns the 4-element output. For global-phase equations, `locals` in
/// `inputs` can be zeroed. For local-phase equations, `locals` should contain
/// the particle's parameter values and living time.
pub fn evaluate(
    eq: &DynamicEquation,
    inputs: &EvalInputs,
    rand: &mut dyn RandProvider,
) -> [f32; 4] {
    let mut registers = vec![0.0f32; eq.register_count as usize];

    let get = |reg: &RegRef, registers: &[f32]| -> f32 {
        match *reg {
            RegRef::Temp(i) => registers[i as usize],
            RegRef::External(i) => inputs.externals[i as usize],
            RegRef::Global(i) => {
                debug_assert_eq!(i, 0);
                inputs.global_time
            }
            RegRef::Local(i) => inputs.locals[i as usize],
        }
    };

    for op in &eq.operations {
        // Gather inputs into a temporary buffer (C++ uses max 8)
        let temp_inputs: Vec<f32> = op.inputs.iter().map(|r| get(r, &registers)).collect();

        for (j, out_reg) in op.outputs.iter().enumerate() {
            let RegRef::Temp(idx) = *out_reg else {
                // Should not happen after validation, but be safe.
                continue;
            };

            let value = match op.op_type {
                DynOpType::Constant => {
                    // Attribute[0] holds the float bits as i32
                    let bits = op.attributes.first().copied().unwrap_or(0);
                    f32::from_bits(bits as u32)
                }
                DynOpType::Add => temp_inputs[0] + temp_inputs[1],
                DynOpType::Sub => temp_inputs[0] - temp_inputs[1],
                DynOpType::Mul => temp_inputs[0] * temp_inputs[1],
                DynOpType::Div => temp_inputs[0] / temp_inputs[1],
                DynOpType::Mod => temp_inputs[0] % temp_inputs[1],
                DynOpType::UnaryAdd => temp_inputs[0],
                DynOpType::UnarySub => -temp_inputs[0],
                // C++ uses `tempInputs[j]` for Sine/Cos (output index, not input 0)
                DynOpType::Sine => temp_inputs[j].sin(),
                DynOpType::Cos => temp_inputs[j].cos(),
                DynOpType::Rand => rand.rand(),
                // C++ uses `tempInputs[j]` for the seed
                DynOpType::RandWithSeed => rand.rand_with_seed(temp_inputs[j]),
                DynOpType::Step => {
                    let edge = temp_inputs[0];
                    let x = temp_inputs[1];
                    if x >= edge {
                        1.0
                    } else {
                        0.0
                    }
                }
            };

            registers[idx as usize] = value;
        }
    }

    // Read final outputs
    let mut result = [0.0f32; 4];
    for (i, out_reg) in eq.output_registers.iter().enumerate() {
        result[i] = get(out_reg, &registers);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::dynamic_equation::*;

    /// Helper to build bytecode from structured data.
    fn build_bytecode(
        version: i32,
        phase: i32,
        register_count: i32,
        output_regs: [i32; 4],
        ops: &[(i32, &[i32], &[i32], &[i32])], // (type, inputs, outputs, attributes)
    ) -> Vec<u8> {
        let mut buf = Vec::new();
        let push_i32 = |buf: &mut Vec<u8>, v: i32| buf.extend_from_slice(&v.to_le_bytes());

        push_i32(&mut buf, version);
        push_i32(&mut buf, phase);
        push_i32(&mut buf, register_count);
        push_i32(&mut buf, ops.len() as i32);
        for &r in &output_regs {
            push_i32(&mut buf, r);
        }

        for &(op_type, inputs, outputs, attrs) in ops {
            push_i32(&mut buf, op_type);
            push_i32(&mut buf, inputs.len() as i32);
            push_i32(&mut buf, outputs.len() as i32);
            push_i32(&mut buf, attrs.len() as i32);
            for &v in inputs {
                push_i32(&mut buf, v);
            }
            for &v in outputs {
                push_i32(&mut buf, v);
            }
            for &v in attrs {
                push_i32(&mut buf, v);
            }
        }
        buf
    }

    fn float_bits(v: f32) -> i32 {
        i32::from_le_bytes(v.to_le_bytes())
    }

    struct FixedRand(f32);
    impl RandProvider for FixedRand {
        fn rand(&mut self) -> f32 {
            self.0
        }
        fn rand_with_seed(&mut self, _seed: f32) -> f32 {
            self.0
        }
    }

    #[test]
    fn test_decode_constant() {
        // Single constant op: reg0 = 42.0, output[0..3] = reg0
        let data = build_bytecode(
            0,
            0, // Global
            1, // 1 temp register
            [0, 0, 0, 0],
            &[(
                0, // Constant
                &[],
                &[0],
                &[float_bits(42.0)],
            )],
        );

        let eq = decode(&data).unwrap();
        assert_eq!(eq.phase, RunningPhase::Global);
        assert_eq!(eq.register_count, 1);
        assert_eq!(eq.operations.len(), 1);
        assert_eq!(eq.operations[0].op_type, DynOpType::Constant);

        let result = evaluate(&eq, &EvalInputs::default(), &mut FixedRand(0.5));
        assert!((result[0] - 42.0).abs() < 1e-6);
    }

    #[test]
    fn test_decode_add() {
        // reg0 = 10.0, reg1 = 32.0, reg2 = reg0 + reg1
        // output = reg2
        let data = build_bytecode(
            0,
            0,
            3,
            [2, 2, 2, 2],
            &[
                (0, &[], &[0], &[float_bits(10.0)]),  // Constant -> reg0
                (0, &[], &[1], &[float_bits(32.0)]),  // Constant -> reg1
                (1, &[0, 1], &[2], &[]),               // Add reg0+reg1 -> reg2
            ],
        );

        let eq = decode(&data).unwrap();
        let result = evaluate(&eq, &EvalInputs::default(), &mut FixedRand(0.5));
        assert!((result[0] - 42.0).abs() < 1e-6);
    }

    #[test]
    fn test_decode_external_inputs() {
        // reg0 = external[0] + external[1]
        // output = reg0
        let ext0 = 0x1000;
        let ext1 = 0x1001;
        let data = build_bytecode(
            0,
            0,
            1,
            [0, 0, 0, 0],
            &[(1, &[ext0, ext1], &[0], &[])], // Add external[0]+external[1] -> reg0
        );

        let eq = decode(&data).unwrap();
        let inputs = EvalInputs {
            externals: [100.0, 200.0, 0.0, 0.0],
            ..Default::default()
        };
        let result = evaluate(&eq, &inputs, &mut FixedRand(0.5));
        assert!((result[0] - 300.0).abs() < 1e-6);
    }

    #[test]
    fn test_decode_global_time() {
        // reg0 = global_time * 2.0
        let global_time_reg = 0x1100;
        let data = build_bytecode(
            0,
            1, // Local
            2,
            [0, 0, 0, 0],
            &[
                (0, &[], &[1], &[float_bits(2.0)]),             // Constant 2.0 -> reg1
                (3, &[global_time_reg, 1], &[0], &[]),          // Mul global_time * reg1 -> reg0
            ],
        );

        let eq = decode(&data).unwrap();
        assert_eq!(eq.phase, RunningPhase::Local);
        let inputs = EvalInputs {
            global_time: 5.0,
            ..Default::default()
        };
        let result = evaluate(&eq, &inputs, &mut FixedRand(0.5));
        assert!((result[0] - 10.0).abs() < 1e-6);
    }

    #[test]
    fn test_decode_local_registers() {
        // reg0 = local[4] (living time)
        let local4 = 0x1204;
        let data = build_bytecode(
            0,
            1,
            1,
            [0, 0, 0, 0],
            &[(11, &[local4], &[0], &[])], // UnaryAdd local[4] -> reg0 (identity copy)
        );

        let eq = decode(&data).unwrap();
        let inputs = EvalInputs {
            locals: [0.0, 0.0, 0.0, 0.0, 3.5],
            ..Default::default()
        };
        let result = evaluate(&eq, &inputs, &mut FixedRand(0.5));
        assert!((result[0] - 3.5).abs() < 1e-6);
    }

    #[test]
    fn test_step_operation() {
        // Step: if input[1] >= input[0] => 1.0 else 0.0
        // reg0 = 5.0 (edge), reg1 = 10.0 (x), reg2 = step(reg0, reg1)
        let data = build_bytecode(
            0,
            0,
            3,
            [2, 2, 2, 2],
            &[
                (0, &[], &[0], &[float_bits(5.0)]),
                (0, &[], &[1], &[float_bits(10.0)]),
                (50, &[0, 1], &[2], &[]), // Step
            ],
        );

        let eq = decode(&data).unwrap();
        let result = evaluate(&eq, &EvalInputs::default(), &mut FixedRand(0.5));
        assert!((result[0] - 1.0).abs() < 1e-6); // 10 >= 5

        // Test with x < edge
        let data2 = build_bytecode(
            0,
            0,
            3,
            [2, 2, 2, 2],
            &[
                (0, &[], &[0], &[float_bits(10.0)]),
                (0, &[], &[1], &[float_bits(5.0)]),
                (50, &[0, 1], &[2], &[]),
            ],
        );
        let eq2 = decode(&data2).unwrap();
        let result2 = evaluate(&eq2, &EvalInputs::default(), &mut FixedRand(0.5));
        assert!((result2[0] - 0.0).abs() < 1e-6); // 5 < 10
    }

    #[test]
    fn test_trig_operations() {
        use std::f32::consts::FRAC_PI_2;
        // reg0 = pi/2, reg1 = sin(reg0), reg2 = cos(reg0)
        let data = build_bytecode(
            0,
            0,
            3,
            [1, 2, 1, 2],
            &[
                (0, &[], &[0], &[float_bits(FRAC_PI_2)]),
                (21, &[0], &[1], &[]), // Sine
                (22, &[0], &[2], &[]), // Cos
            ],
        );

        let eq = decode(&data).unwrap();
        let result = evaluate(&eq, &EvalInputs::default(), &mut FixedRand(0.5));
        assert!((result[0] - 1.0).abs() < 1e-5); // sin(pi/2) = 1
        assert!((result[1] - 0.0).abs() < 1e-5); // cos(pi/2) ≈ 0
    }

    #[test]
    fn test_sub_div_mod_negate() {
        // reg0=10, reg1=3, reg2=sub, reg3=div, reg4=mod, reg5=negate
        let data = build_bytecode(
            0,
            0,
            6,
            [2, 3, 4, 5],
            &[
                (0, &[], &[0], &[float_bits(10.0)]),
                (0, &[], &[1], &[float_bits(3.0)]),
                (2, &[0, 1], &[2], &[]),  // Sub 10-3=7
                (4, &[0, 1], &[3], &[]),  // Div 10/3≈3.333
                (5, &[0, 1], &[4], &[]),  // Mod 10%3=1
                (12, &[0], &[5], &[]),    // UnarySub -10
            ],
        );

        let eq = decode(&data).unwrap();
        let result = evaluate(&eq, &EvalInputs::default(), &mut FixedRand(0.5));
        assert!((result[0] - 7.0).abs() < 1e-6);
        assert!((result[1] - 10.0 / 3.0).abs() < 1e-5);
        assert!((result[2] - 1.0).abs() < 1e-6);
        assert!((result[3] - (-10.0)).abs() < 1e-6);
    }

    #[test]
    fn test_rand_operation() {
        // reg0 = rand()
        let data = build_bytecode(0, 0, 1, [0, 0, 0, 0], &[(31, &[], &[0], &[])]);

        let eq = decode(&data).unwrap();
        let result = evaluate(&eq, &EvalInputs::default(), &mut FixedRand(0.75));
        assert!((result[0] - 0.75).abs() < 1e-6);
    }

    #[test]
    fn test_output_reads_from_non_temp() {
        // Output register points directly to external[2]
        let ext2 = 0x1002;
        let data = build_bytecode(0, 0, 0, [ext2, ext2, ext2, ext2], &[]);

        let eq = decode(&data).unwrap();
        let inputs = EvalInputs {
            externals: [0.0, 0.0, 99.0, 0.0],
            ..Default::default()
        };
        let result = evaluate(&eq, &inputs, &mut FixedRand(0.5));
        assert!((result[0] - 99.0).abs() < 1e-6);
    }

    #[test]
    fn test_error_header_too_short() {
        let data = vec![0u8; 10];
        assert!(matches!(
            decode(&data),
            Err(DecodeError::HeaderTooShort { len: 10 })
        ));
    }

    #[test]
    fn test_error_invalid_phase() {
        // register_count=0 makes output reg 0 invalid, so build the header manually.
        let mut buf = Vec::new();
        let push = |buf: &mut Vec<u8>, v: i32| buf.extend_from_slice(&v.to_le_bytes());
        push(&mut buf, 0);  // version
        push(&mut buf, 99); // invalid phase
        push(&mut buf, 1);  // register_count
        push(&mut buf, 0);  // operator_count
        push(&mut buf, 0);  // output[0]
        push(&mut buf, 0);  // output[1]
        push(&mut buf, 0);  // output[2]
        push(&mut buf, 0);  // output[3]
        assert!(matches!(
            decode(&buf),
            Err(DecodeError::InvalidPhase { value: 99 })
        ));
    }

    #[test]
    fn test_error_invalid_opcode() {
        let data = build_bytecode(
            0,
            0,
            1,
            [0, 0, 0, 0],
            &[(999, &[], &[0], &[float_bits(1.0)])],
        );
        assert!(matches!(
            decode(&data),
            Err(DecodeError::InvalidOpcode {
                value: 999,
                op_index: 0
            })
        ));
    }

    #[test]
    fn test_error_trailing_bytes() {
        let mut data = build_bytecode(0, 0, 1, [0, 0, 0, 0], &[]);
        data.extend_from_slice(&[0, 0, 0, 0]); // Extra 4 bytes
        assert!(matches!(
            decode(&data),
            Err(DecodeError::TrailingBytes { remaining: 4 })
        ));
    }

    #[test]
    fn test_no_ops_identity() {
        // 0 operators, output reads directly from temp reg 0 (initialized to 0.0)
        let data = build_bytecode(0, 0, 1, [0, 0, 0, 0], &[]);
        let eq = decode(&data).unwrap();
        let result = evaluate(&eq, &EvalInputs::default(), &mut FixedRand(0.5));
        assert_eq!(result, [0.0, 0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_multi_output_different_regs() {
        // reg0=1, reg1=2, reg2=3, reg3=4 — output each to a different slot
        let data = build_bytecode(
            0,
            0,
            4,
            [0, 1, 2, 3],
            &[
                (0, &[], &[0], &[float_bits(1.0)]),
                (0, &[], &[1], &[float_bits(2.0)]),
                (0, &[], &[2], &[float_bits(3.0)]),
                (0, &[], &[3], &[float_bits(4.0)]),
            ],
        );
        let eq = decode(&data).unwrap();
        let result = evaluate(&eq, &EvalInputs::default(), &mut FixedRand(0.5));
        assert!((result[0] - 1.0).abs() < 1e-6);
        assert!((result[1] - 2.0).abs() < 1e-6);
        assert!((result[2] - 3.0).abs() < 1e-6);
        assert!((result[3] - 4.0).abs() < 1e-6);
    }
}
