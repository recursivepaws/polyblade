use crate::render::color::RGBA;

#[derive(Debug, Clone)]
pub struct Palette {
    pub colors: Vec<RGBA>,
}

impl From<Palette> for Vec<wgpu::Color> {
    fn from(val: Palette) -> Self {
        val.colors
            .into_iter()
            .map(Into::<wgpu::Color>::into)
            .collect()
    }
}

impl Palette {
    fn new(colors: &[&str]) -> Self {
        Self {
            colors: colors.iter().map(|&c| c.try_into().unwrap()).collect(),
        }
    }

    pub fn polyblade() -> Self {
        Self::new(&[
            "#48845A", "#A3BA70", "#335145", "#FEF086", "#5F9BFC", "#F4A4E7", "#AA89BE",
        ])
    }

    // https://lospec.com/palette-list/desatur8
    pub fn desatur8() -> Self {
        Self::new(&[
            "#f0f0eb", "#ffff8f", "#7be098", "#849ad8", "#e8b382", "#d8828e", "#a776c1", "#545155",
        ])
    }

    pub fn clement() -> Self {
        Self::new(&[
            "#639bff", "#8854f3", "#ff79ae", "#ff8c5c", "#fff982", "#63ffba",
        ])
    }
    pub fn clement_extended() -> Self {
        Self::new(&[
            "#8854f3", "#fff982", "#639bff", "#ff8c5c", "#63ffba", "#ff79ae", "#70f3ff",
        ])
    }

    pub fn dream_haze() -> Self {
        Self::new(&[
            "#3c42c4", "#6e51c8", "#a065cd", "#ce79d2", "#d68fb8", "#dda2a3", "#eac4ae", "#f4dfbe",
        ])
    }
}
