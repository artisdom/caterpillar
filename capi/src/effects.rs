use tokio::sync::mpsc;

pub struct EffectsTx {
    pub inner: mpsc::UnboundedSender<DisplayEffect>,
}

impl EffectsTx {
    pub fn send(&self, effect: DisplayEffect) {
        // This produces an error, if the other end has hung up, which happens
        // during shutdown. We can safely ignore that.
        let _ = self.inner.send(effect);
    }
}

pub type EffectsRx = mpsc::UnboundedReceiver<DisplayEffect>;

#[derive(Debug)]
pub enum DisplayEffect {
    SubmitTiles { reply: mpsc::UnboundedSender<()> },
}
