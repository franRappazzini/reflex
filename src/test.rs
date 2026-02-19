mod tests {
    use mollusk_svm::{result::Check, Mollusk};
    use solana_address::address;
    use solana_instruction::Instruction;

    #[test]
    fn test_hello_world() {
        let program_id = address!("AxXTVNh3eDefaL8F6RVXMKXss77tgUi8MZHQAGrQX5db");
        let mollusk = Mollusk::new(&program_id, "target/deploy/reflex");

        let instruction = Instruction::new_with_bytes(program_id, &[], vec![]);

        mollusk.process_and_validate_instruction(&instruction, &[], &[Check::success()]);
    }
}
