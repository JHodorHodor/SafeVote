pub trait ShareReceiver<Msg> {
    fn recv(&mut self) -> Msg;
}