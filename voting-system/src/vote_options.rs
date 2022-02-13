#[derive(Clone)]
pub(crate) struct VoteOptions {
    id: usize,
    number_of_voters: usize,
    vote_threshold: usize,
    number_of_options: usize,
}

impl VoteOptions {
    pub(crate) fn new(id: usize, number_of_voters: usize, vote_threshold: usize, number_of_options: usize) -> Self {
        VoteOptions {
            id, number_of_voters, vote_threshold, number_of_options
        }
    }

    pub(crate) fn get_id(&self) -> usize {
        self.id
    }

    pub(crate) fn get_number_of_voters(&self) -> usize {
        self.number_of_voters
    }

    pub(crate) fn get_vote_threshold(&self) -> usize {
        self.vote_threshold
    }

    pub(crate) fn get_number_of_options(&self) -> usize {
        self.number_of_options
    }
}