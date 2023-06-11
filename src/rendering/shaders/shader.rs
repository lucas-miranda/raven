use std::{
    hash::{ Hash, Hasher },
    fmt::Display
};

use super::ShaderInfo;

// TODO  change visibility to crate only
//       an user should not mess with shader id
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct ShaderId(u32);

impl ShaderId {
    pub(super) fn next(&mut self) {
        self.0 += 1;
    }
}

impl Display for ShaderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

pub struct Shader {
    id: ShaderId,
}

impl Shader {
    pub(super) fn new(id: ShaderId) -> Self {
        Self {
            id,
        }
    }

    pub(super) fn id(&self) -> &ShaderId {
        &self.id
    }
}

impl Hash for Shader {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl ShaderInfo for Shader {
    fn id(&self) -> ShaderId {
        self.id
    }
}

impl AsRef<dyn ShaderInfo + 'static> for Shader {
    fn as_ref(&self) -> &(dyn ShaderInfo + 'static) {
        self
    }
}
