use crate::agent::{traits::RecordedAgentSet, AgentSet};
use alloy_primitives::Address;

pub type AgentSetRef = Box<dyn AgentSet>;
pub struct AgentSetVec(pub Vec<AgentSetRef>);

impl Default for AgentSetVec {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentSetVec {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push_agent_set<A: AgentSet + 'static>(&mut self, agent_set: A) {
        self.0.push(Box::new(agent_set))
    }
    pub fn get_addresses(&self, index: usize) -> Vec<Address> {
        self.0.get(index).unwrap().get_addresses()
    }
    pub fn get_records<R, A: RecordedAgentSet<R> + 'static>(
        &mut self,
        index: usize,
    ) -> Vec<Vec<R>> {
        let x = self.0.get_mut(index).unwrap();
        let x = match x.as_mut_any().downcast_mut::<A>() {
            Some(a) => a,
            None => panic!("Incorrect downcast when retrieving records"),
        };
        x.take_records()
    }
}
