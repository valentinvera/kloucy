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

/// Lucide icons the component library does not ship, served at the same
/// `icons/*.svg` paths so `gpui::svg()` can find them next to the rest.
const EXTRA_ICONS: [(&str, &[u8]); 5] = [
    (
        "icons/paperclip.svg",
        include_bytes!("../assets/icons/paperclip.svg"),
    ),
    (
        "icons/git-branch.svg",
        include_bytes!("../assets/icons/git-branch.svg"),
    ),
    // Agent marks. The marks belong to their owners: OpenAI's blossom comes
    // from Wikimedia Commons, OpenCode's from Simple Icons; both are
    // wordmarks in a single ink, so the UI tints them like the icon set.
    (
        "icons/agents/openai.svg",
        include_bytes!("../assets/icons/agents/openai.svg"),
    ),
    (
        "icons/agents/opencode.svg",
        include_bytes!("../assets/icons/agents/opencode.svg"),
    ),
    // Claude's mark carries its own colour, which an SVG mask would flatten,
    // so it ships as a small transparent PNG instead.
    (
        "icons/agents/claude.png",
        include_bytes!("../assets/icons/agents/claude.png"),
    ),
];

/// Geist plus the Lucide icons [gpui_component::IconName] resolves to
/// `icons/*.svg`.
pub struct Assets;

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some((_, icon)) = EXTRA_ICONS.iter().find(|(name, _)| *name == path) {
            return Ok(Some(Cow::Borrowed(*icon)));
        }

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

    #[test]
    fn embeds_the_agent_marks() {
        let png = Assets
            .load("icons/agents/claude.png")
            .expect("the mark should load")
            .expect("the mark should be embedded");

        assert!(png.starts_with(&[0x89, b'P', b'N', b'G']));
        assert!(png.len() > 1024);
    }

    #[test]
    fn serves_the_icons_the_component_library_lacks() {
        for (name, _) in EXTRA_ICONS {
            let icon = Assets.load(name).expect("an icon should load");

            assert!(icon.is_some(), "{name} should be served");
        }
    }
}
