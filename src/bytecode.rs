use crate::{Instruction, Result, UfoError};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Bytecode {
    instructions: Vec<Instruction>,
    is_64bit: bool,
}

impl Bytecode {
    pub fn new(is_64bit: bool) -> Self {
        Self {
            instructions: Vec::new(),
            is_64bit,
        }
    }

    pub fn from_instructions(instructions: Vec<Instruction>, is_64bit: bool) -> Self {
        Self {
            instructions,
            is_64bit,
        }
    }

    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    pub fn is_64bit(&self) -> bool {
        self.is_64bit
    }

    pub fn word_size(&self) -> usize {
        if self.is_64bit { 8 } else { 4 }
    }

    pub fn push(&mut self, instruction: Instruction) {
        self.instructions.push(instruction);
    }

    pub fn emit_bytes(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        for instruction in &self.instructions {
            if self.is_64bit {
                out.extend(instruction.encode64()?);
            } else {
                out.extend(instruction.encode32()?);
            }
        }
        Ok(out)
    }

    pub fn emit_u32_words(&self) -> Result<Vec<u32>> {
        let bytes = self.emit_bytes()?;
        Ok(bytes
            .chunks_exact(4)
            .map(|chunk| u32::from_le_bytes(chunk.try_into().unwrap()))
            .collect())
    }

    pub fn emit_u64_words(&self) -> Result<Vec<u64>> {
        let bytes = self.emit_bytes()?;
        Ok(bytes
            .chunks_exact(8)
            .map(|chunk| u64::from_le_bytes(chunk.try_into().unwrap()))
            .collect())
    }

    pub fn word_count(&self) -> usize {
        self.instructions
            .iter()
            .map(Instruction::encoded_word_count)
            .sum()
    }

    pub fn chunked(&self, chunk_words: usize) -> Result<Self> {
        if chunk_words < 2 {
            return Err(UfoError::InvalidBytecodeChunkSize);
        }

        let mut out = Vec::new();
        let mut current_words = 0usize;
        for instruction in &self.instructions {
            let words = instruction.encoded_word_count();
            if words > chunk_words - 1 {
                return Err(UfoError::InstructionTooLargeForChunk {
                    name: instruction.name.clone(),
                    words,
                    chunk_words,
                });
            }

            if current_words > 0 && current_words + words > chunk_words - 1 {
                while current_words < chunk_words - 1 {
                    out.push(Instruction::noop("chunk_pad"));
                    current_words += 1;
                }
                out.push(Instruction::next_offset("next_offset"));
                current_words = 0;
            }

            out.push(instruction.clone());
            current_words += words;
        }

        Ok(Self::from_instructions(out, self.is_64bit))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emits_legacy_compatible_32_bit_words() {
        let bytecode = Bytecode::from_instructions(
            vec![
                Instruction::drift("d1", 1.2, true),
                Instruction::kick("k1", vec![0.0, 0.1], Vec::new(), false, false),
                Instruction::rewind("rw"),
            ],
            false,
        );
        assert_eq!(bytecode.emit_bytes().unwrap().len(), 24);
        assert_eq!(bytecode.emit_u32_words().unwrap().len(), 6);
    }

    #[test]
    fn emits_legacy_compatible_64_bit_words() {
        let bytecode = Bytecode::from_instructions(vec![Instruction::rewind("rw")], true);
        assert_eq!(
            bytecode.emit_bytes().unwrap(),
            vec![255, 0, 128, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn chunks_bytecode_with_next_offsets_and_padding() {
        let bytecode = Bytecode::from_instructions(
            vec![
                Instruction::drift("d1", 1.0, false),
                Instruction::drift("d2", 2.0, false),
                Instruction::rewind("rw"),
            ],
            false,
        );
        let chunked = bytecode.chunked(4).unwrap();
        assert!(
            chunked
                .instructions()
                .iter()
                .any(|i| i.kind == "next_offset")
        );
        assert_eq!(chunked.word_count(), 7);
    }
}
