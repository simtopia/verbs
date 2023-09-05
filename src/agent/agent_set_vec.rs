use crate::agent::{traits::RecordedAgentSet, AgentSet};
use ethers_core::types::Address;

pub type AgentSetRef<'a> = Box<&'a mut dyn AgentSet>;
pub struct AgentSetVec<'a>(pub Vec<AgentSetRef<'a>>);

impl<'a> AgentSetVec<'a> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn push_agent_set<A: AgentSet>(&mut self, agent_set: &'a mut A) {
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
