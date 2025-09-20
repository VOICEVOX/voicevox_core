use std::fmt::Display;

use easy_ext::ext;

use crate::{CharacterMeta, StyleId, StyleMeta};

pub trait VoiceSpec {
    type Priority: Ord;
    fn display(&self) -> impl Display;
    fn priority(&mut self, voice: Voice<'_>) -> Self::Priority;
}

impl<F: FnMut(Voice<'_>) -> T, T: Ord> VoiceSpec for F {
    type Priority = T;

    fn display(&self) -> impl Display {
        "<custom>"
    }

    fn priority(&mut self, voice: Voice<'_>) -> Self::Priority {
        self(voice)
    }
}

impl VoiceSpec for Voice<'_> {
    type Priority = bool;

    fn display(&self) -> impl Display {
        self
    }

    fn priority(&mut self, voice: Voice<'_>) -> Self::Priority {
        voice.style.id == self.style.id
    }
}

impl VoiceSpec for u32 {
    type Priority = bool;

    fn display(&self) -> impl Display {
        self
    }

    fn priority(&mut self, voice: Voice<'_>) -> Self::Priority {
        voice.style.id == StyleId(*self)
    }
}

impl VoiceSpec for StyleId {
    type Priority = bool;

    fn display(&self) -> impl Display {
        self
    }

    fn priority(&mut self, voice: Voice<'_>) -> Self::Priority {
        voice.style.id == *self
    }
}

impl<'metas> VoiceSpec for (&'metas str, &'metas str) {
    type Priority = bool;

    fn display(&self) -> impl Display {
        let (character_name, style_name) = self;
        format!("{character_name}（{style_name}）")
    }

    fn priority(&mut self, voice: Voice<'_>) -> Self::Priority {
        (&*voice.character.name, &*voice.style.name) == *self
    }
}

#[derive(Clone, Copy, Debug, derive_more::Display)]
#[display("{}（{}）", character.name, style.name)]
#[non_exhaustive]
pub struct Voice<'metas> {
    pub style: &'metas StyleMeta,
    pub character: &'metas CharacterMeta,
}

#[ext(VoiceSpecExt)]
impl<S: VoiceSpec> S {
    pub(crate) fn as_mut(&mut self) -> impl VoiceSpec + use<'_, S> {
        return VoiceSpecAsMut(self);

        struct VoiceSpecAsMut<'a, S>(&'a mut S);

        impl<S: VoiceSpec> VoiceSpec for VoiceSpecAsMut<'_, S> {
            type Priority = S::Priority;

            fn display(&self) -> impl Display {
                self.0.display()
            }

            fn priority(&mut self, voice: Voice<'_>) -> Self::Priority {
                self.0.priority(voice)
            }
        }
    }
}
