//! Everything the app loads through GPUI's asset source: the UI font and the
//! icon set the component library asks for.

use std::borrow::Cow;

use gpui::{AssetSource, Result, SharedString};

/// Geist ships as separate cuts, so the whole app can use one family without
/// requiring it to be installed on the machine.
const GEIST: [&[u8]; 4] = [
    include_bytes!("../assets/fonts/Geist_400Regular.ttf"),
    include_bytes!("../assets/fonts/Geist_500Medium.ttf"),
    include_bytes!("../assets/fonts/Geist_600SemiBold.ttf"),
    include_bytes!("../assets/fonts/Geist_700Bold.ttf"),
];

/// The family name the embedded cuts register under, per their font metadata.
pub const FONT_FAMILY: &str = "Geist";

/// Geist plus the Lucide icons [gpui_component::IconName] resolves to
/// `icons/*.svg`.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        gpui_component_assets::Assets.load(path)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        gpui_component_assets::Assets.list(path)
    }
}

/// The embedded Geist cuts, for `TextSystem::add_fonts`.
pub fn geist_fonts() -> Vec<Cow<'static, [u8]>> {
    GEIST.iter().map(|font| Cow::Borrowed(*font)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ships_every_geist_cut() {
        let fonts = geist_fonts();

        assert_eq!(fonts.len(), GEIST.len());
        assert!(fonts.iter().all(|font| font.len() > 32_768));
    }

    #[test]
    fn keeps_serving_the_icon_set() {
        let icon = Assets.load("icons/bot.svg").expect("an icon should load");

        assert!(icon.is_some());
        assert!(Assets.load("icons/not-a-real-icon.svg").is_err());
    }
}
