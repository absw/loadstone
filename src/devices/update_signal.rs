/// Indicates the state of an update signal.
#[derive(Copy, Clone, Debug)]
pub enum UpdatePlan {
    /// Do not update.
    None,

    /// Allow updates, if one is available.
    Any,

    /// Update from a specific image.
    Index(u8),
}

pub trait ReadUpdateSignal {
    fn read_update_plan(&self) -> UpdatePlan;
}

pub trait WriteUpdateSignal {
    fn write_update_plan(&mut self, plan: UpdatePlan);
}
