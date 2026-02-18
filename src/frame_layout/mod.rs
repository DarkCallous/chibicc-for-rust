use crate::resolver::ResolvedCrate;
use crate::resolver::*;
use std::collections::HashMap;
use std::fmt::Display;

pub struct FrameLayout {
    pub slots: HashMap<ObjId, usize>,
    pub frame_size: usize,
}

pub struct FrameLayouts {
    pub fns: HashMap<ObjId, FrameLayout>,
}

#[derive(Default)]
pub struct FrameBuilder {}

impl FrameBuilder {
    pub fn new() -> FrameBuilder {
        FrameBuilder {}
    }

    pub fn build(&self, crat: &ResolvedCrate) -> FrameLayouts {
        let fns = crat
            .fn_info
            .values()
            .map(|func| (func.fn_id, self.build_fn(func)))
            .collect();
        FrameLayouts { fns }
    }

    fn build_fn(&self, func: &FnInfo) -> FrameLayout {
        let mut offset = 0;
        let layout = func
            .params
            .iter()
            .chain(func.locals.iter())
            .map(|vid| {
                offset += 8;
                (*vid, offset)
            })
            .collect();
        FrameLayout {
            slots: layout,
            frame_size: offset,
        }
    }
}
