use druid::Selector;

pub(crate) const VOTE: Selector<Vec<bool>> = Selector::new("app.vote");

pub(crate) const VOTE_OUTPUT: Selector<Vec<bool>> = Selector::new("app.vote_output");
