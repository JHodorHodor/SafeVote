pub trait ShareSender<Msg> {
    fn send(&mut self, msg: Msg);
}