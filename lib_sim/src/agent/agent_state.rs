use crate::contract::Call;
use sim_macros::SimState;

pub trait SimState {
    fn call_agents(&mut self) -> Vec<Call>;
}

pub trait AgentSet {
    fn call(&mut self) -> Vec<Call>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_primitives::Address;

    struct DummyAgent {
        v: bool,
    }

    impl AgentSet for DummyAgent {
        fn call(&mut self) -> Vec<Call> {
            vec![Call {
                function_name: "foo",
                callee: Address::ZERO,
                transact_to: Address::ZERO,
                args: Vec::default(),
                checked: self.v,
            }]
        }
    }

    #[test]
    fn test_macro() {
        #[derive(SimState)]
        struct TestState {
            a: DummyAgent,
            b: DummyAgent,
        }

        let mut x = TestState {
            a: DummyAgent { v: true },
            b: DummyAgent { v: false },
        };

        let calls = x.call_agents();

        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].checked, true);
        assert_eq!(calls[1].checked, false);
    }
}
