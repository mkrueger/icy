//! Tests for the theme module.

#[cfg(test)]
mod tests {
    use crate::theme::{Component, CornerRadii, Layer, Palette, Spacing, Theme};

    #[test]
    fn test_dark_theme() {
        let theme = Theme::dark();
        assert!(theme.is_dark);
        assert_eq!(theme.name, "Dark");
    }

    #[test]
    fn test_light_theme() {
        let theme = Theme::light();
        assert!(!theme.is_dark);
        assert_eq!(theme.name, "Light");
    }

    #[test]
    fn test_default_theme_is_dark() {
        let theme = Theme::default();
        assert!(theme.is_dark);
    }

    #[test]
    fn test_palette_accent() {
        let dark = Palette::dark();
        let light = Palette::light();

        // Both should have valid accent colors
        let dark_accent = dark.accent();
        let light_accent = light.accent();

        assert!(dark_accent.r >= 0.0 && dark_accent.r <= 1.0);
        assert!(light_accent.r >= 0.0 && light_accent.r <= 1.0);
    }

    #[test]
    fn test_spacing_default() {
        let spacing = Spacing::default();

        assert_eq!(spacing.none, 0);
        assert!(spacing.xs < spacing.s);
        assert!(spacing.s < spacing.m);
        assert!(spacing.m < spacing.l);
        assert!(spacing.l < spacing.xl);
    }

    #[test]
    fn test_corner_radii_default() {
        let radii = CornerRadii::default();

        assert_eq!(radii.radius_0, [0.0; 4]);
        // Other radii should be increasing
    }

    #[test]
    fn test_container_layers() {
        let theme = Theme::dark();

        // Background should be different from primary
        assert_ne!(theme.background.base, theme.primary.base);
        // Primary should be different from secondary
        assert_ne!(theme.primary.base, theme.secondary.base);
    }

    #[test]
    fn test_component_generation() {
        let palette = Palette::dark();
        let component = Component::accent(&palette, true);

        // Should have valid colors
        assert!(component.base.r >= 0.0 && component.base.r <= 1.0);
        assert!(component.hover.r >= 0.0 && component.hover.r <= 1.0);
        assert!(component.pressed.r >= 0.0 && component.pressed.r <= 1.0);
    }

    #[test]
    fn test_layer_selection() {
        let theme = Theme::dark();

        let bg = theme.container(Layer::Background);
        let primary = theme.container(Layer::Primary);
        let secondary = theme.container(Layer::Secondary);

        assert_eq!(bg.base, theme.background.base);
        assert_eq!(primary.base, theme.primary.base);
        assert_eq!(secondary.base, theme.secondary.base);
    }

    #[test]
    fn test_on_colors() {
        let theme = Theme::dark();

        // Text colors should be valid
        let on_bg = theme.on_background();
        let on_primary = theme.on_primary();
        let on_secondary = theme.on_secondary();

        assert!(on_bg.r >= 0.0 && on_bg.r <= 1.0);
        assert!(on_primary.r >= 0.0 && on_primary.r <= 1.0);
        assert!(on_secondary.r >= 0.0 && on_secondary.r <= 1.0);
    }

    #[test]
    fn test_theme_from_palette() {
        let palette = Palette::light();
        let theme = Theme::from_palette(palette.clone(), false);

        assert!(!theme.is_dark);
        assert_eq!(theme.palette.neutral_0, palette.neutral_0);
    }
}
