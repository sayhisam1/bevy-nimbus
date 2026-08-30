//! User-facing cloud components and quality settings.

use bevy::{camera::Hdr, prelude::*, render::extract_component::ExtractComponent};

/// Selects a canonical cloud-density and color profile.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CloudPreset {
    /// Mostly clear sky with sparse low clouds.
    Clear,
    /// Substantial separated cumulus forms.
    #[default]
    Scattered,
    /// A dense, muted cloud deck.
    Overcast,
    /// A dark, high-coverage storm deck.
    Storm,
}

/// Named resolved values for one cloud preset or blend.
#[derive(Clone, Copy, Debug, PartialEq)]
#[doc(hidden)]
pub struct CloudProfile {
    pub density: f32,
    pub base_color: Vec3,
}

impl CloudPreset {
    #[must_use]
    #[doc(hidden)]
    pub const fn profile(self) -> CloudProfile {
        match self {
            Self::Clear => CloudProfile {
                density: 0.14,
                base_color: Vec3::ONE,
            },
            Self::Scattered => CloudProfile {
                density: 0.52,
                base_color: Vec3::ONE,
            },
            Self::Overcast => CloudProfile {
                density: 0.9,
                base_color: Vec3::new(0.78, 0.82, 0.88),
            },
            Self::Storm => CloudProfile {
                density: 0.98,
                base_color: Vec3::new(0.12, 0.15, 0.22),
            },
        }
    }
}

/// Ambient cloud lighting owned by a cloud-volume entity.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct NimbusAmbientLight {
    /// Sky-light multiplier.
    pub strength: f32,
    /// Sky-light color.
    pub color: Color,
}

impl Default for NimbusAmbientLight {
    fn default() -> Self {
        Self {
            strength: 0.8,
            color: Color::srgb(0.55, 0.68, 0.95),
        }
    }
}

/// Marks an entity that owns one volumetric cloud field.
#[derive(Component, Clone, Copy, Debug, Default)]
#[require(CloudSettings, CloudPreset, NimbusSampling, NimbusAmbientLight)]
pub struct NimbusCloudVolume;

/// Connects a camera to its cloud-volume entity.
#[derive(Component, ExtractComponent, Clone, Copy, Debug, PartialEq, Eq)]
#[require(Camera3d, Hdr, Msaa::Off)]
pub struct NimbusCloudView {
    /// Entity carrying [`NimbusCloudVolume`] and [`CloudSettings`].
    #[entities]
    pub volume: Entity,
}

impl NimbusCloudView {
    /// Connects a camera to `volume`.
    #[must_use]
    pub const fn new(volume: Entity) -> Self {
        Self { volume }
    }
}

/// Volumetric sampling controls owned by a cloud-volume entity.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct NimbusSampling {
    /// View-march quality (`24..=64`; lower values become 24).
    pub view_steps: u32,
    /// Self-shadow samples (`0`, `2`, or `4`; intermediate values normalize upward).
    pub light_steps: u32,
    /// Reduced-resolution divisor (`2..=8`).
    pub resolution_divisor: u32,
}

impl Default for NimbusSampling {
    fn default() -> Self {
        Self {
            view_steps: 48,
            light_steps: 2,
            resolution_divisor: 4,
        }
    }
}

/// Controls one volumetric cloud field.
#[derive(Component, Clone, Debug, PartialEq)]
pub struct CloudSettings {
    /// Cloud-volume lower bound, in world units.
    pub base_altitude: f32,
    /// Vertical volume thickness in world units; values below 800 become 800.
    pub thickness: f32,
    /// Horizontal X/Z field movement, in world units per second.
    pub movement: Vec2,
    /// Cloud coverage control (`0` clear to `1` overcast).
    pub density: f32,
    /// Positive world-to-texture tiling multiplier.
    pub scale: f32,
}

impl Default for CloudSettings {
    fn default() -> Self {
        Self {
            base_altitude: 1500.0,
            thickness: 1000.0,
            movement: Vec2::new(6.0, 0.0),
            density: 0.52,
            scale: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NimbusCloudView;
    use bevy::{camera::Hdr, prelude::*, render::view::Msaa};

    #[test]
    fn cloud_view_requires_an_hdr_single_sample_camera() {
        let mut world = World::new();
        let volume = world.spawn_empty().id();
        let view = world.spawn(NimbusCloudView::new(volume)).id();

        assert!(world.entity(view).contains::<Hdr>());
        assert_eq!(world.entity(view).get::<Msaa>(), Some(&Msaa::Off));
    }
}
