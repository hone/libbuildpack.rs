use std::ops::Deref;

use crate::error::Error;

const ENV_VAR: &str = "CNB_STACK_ID";

pub struct Stack(String);

impl Deref for Stack {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

impl Stack {
    pub fn new() -> Result<Self, Error> {
        let stack = std::env::var(ENV_VAR)?;

        Ok(Stack { 0: stack })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_reads_stack_from_env() {
        let old_env = std::env::var_os("CNB_STACK_ID");

        std::env::set_var("CNB_STACK_ID", "aspen");
        let stack = Stack::new();

        if let Some(cnb_stack_id) = old_env {
            std::env::set_var("CNB_STACK_ID", cnb_stack_id);
        } else {
            std::env::remove_var("CNB_STACK_ID");
        }

        assert!(stack.is_ok());
        if let Ok(stack) = stack {
            assert_eq!("aspen", *stack);
        }
    }
}
