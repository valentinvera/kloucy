//! Picking a monospace family from what the platform actually offers.
//!
//! Kloucy renders code, commands, paths and terminal output in monospace. Since
//! the resolved family has to exist on the machine, the family is chosen from
//! the installed fonts instead of hard-coding one per platform.

/// Preferred families, in order. The list spans macOS, Linux and Windows.
const MONO_PREFERENCES: [&str; 7] = [
    "Geist Mono",
    "Menlo",
    "DejaVu Sans Mono",
    "Liberation Mono",
    "Noto Sans Mono",
    "Consolas",
    "Courier New",
];

/// The first preferred family the platform offers, if any.
pub fn pick_mono_family(available: &[String]) -> Option<&'static str> {
    MONO_PREFERENCES
        .iter()
        .find(|family| available.iter().any(|name| name == *family))
        .copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn installed(families: &[&str]) -> Vec<String> {
        families
            .iter()
            .map(|family| (*family).to_string())
            .collect()
    }

    #[test]
    fn prefers_geist_mono_when_it_is_installed() {
        assert_eq!(
            pick_mono_family(&installed(&["Menlo", "Geist Mono"])),
            Some("Geist Mono")
        );
    }

    #[test]
    fn finds_what_macos_and_linux_offer() {
        assert_eq!(pick_mono_family(&installed(&["Menlo"])), Some("Menlo"));
        assert_eq!(
            pick_mono_family(&installed(&["DejaVu Sans Mono"])),
            Some("DejaVu Sans Mono")
        );
    }

    #[test]
    fn ignores_families_that_are_not_monospace() {
        assert_eq!(
            pick_mono_family(&installed(&["Geist", "Helvetica", "Arial"])),
            None
        );
        assert_eq!(pick_mono_family(&[]), None);
    }
}
