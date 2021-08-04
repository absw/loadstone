use marker_blanket::marker_blanket;

/// Indicates the state of an update signal.
#[derive(Copy, Clone, Debug)]
pub enum UpdatePlan {
    /// Do not update.
    None,
    /// Allow updates, if one is available.
    Any,
    /// Update from a specific image.
    Index(u8),
    /// Attempt to update through serial recovery once.
    Serial,
}

pub trait ReadUpdateSignal {
    fn read_update_plan(&self) -> UpdatePlan;
}

pub trait WriteUpdateSignal {
    fn write_update_plan(&mut self, plan: UpdatePlan);
}

#[marker_blanket]
pub trait UpdatePlanner: ReadUpdateSignal + WriteUpdateSignal {}
