pub trait Automaton {
    type State;
}
pub trait Pushdown : Automaton {
    fn push_state(&mut self, state: Self::State);
    fn pop_state(&mut self);
}