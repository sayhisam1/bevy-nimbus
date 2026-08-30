//! ECS components, shader functions, and cloud shadows for Nimbus lighting.

mod shadow;

#[doc(hidden)]
pub use shadow::NimbusCloudShadowPlugin;
pub use shadow::{NimbusCloudShadowMap, NimbusCloudShadowSource};

#[doc(hidden)]
pub const LIGHTING_WGSL: &str = include_str!("lighting.wgsl");

use bevy::{light::light_consts::lux, prelude::*};
use nimbus_core::DirectionalLightUniform;
pub use nimbus_core::{MAX_DIRECTIONAL_LIGHTS, NimbusAmbientLight};

/// Mean Earth radius used for the geometric horizon dip.
const EARTH_RADIUS_METERS: f32 = 6_371_000.0;
/// Angular width for the solar disc plus a small allowance for atmospheric refraction.
const HORIZON_SOFTNESS_RADIANS: f32 = 0.75_f32.to_radians();
/// Effective clear-air Rayleigh optical depth at representative red, green, and blue wavelengths.
/// The absolute scale is intentionally chromatic-only: light strength owns achromatic extinction.
const RAYLEIGH_TINT_OPTICAL_DEPTH: Vec3 = Vec3::new(0.0068, 0.0161, 0.04);

/// Role of a directional light in a cloud volume.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum NimbusCloudLightKind {
    /// Drives world shadows.
    Primary,
    /// Illuminates and self-shadows clouds without producing world shadows.
    Secondary,
}

/// Includes a Bevy [`DirectionalLight`] entity in one cloud volume's lighting.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
#[require(DirectionalLight, Transform)]
pub struct NimbusCloudLight {
    /// Target cloud-volume entity.
    #[entities]
    volume: Entity,
    /// Role of this directional light.
    kind: NimbusCloudLightKind,
    /// Optional cloud-light strength override. `None` derives strength from Bevy illuminance.
    strength_override: Option<f32>,
}

impl NimbusCloudLight {
    /// Adds a directional light with an explicit role to `volume`.
    #[must_use]
    pub const fn new(volume: Entity, kind: NimbusCloudLightKind) -> Self {
        Self {
            volume,
            kind,
            strength_override: None,
        }
    }

    /// Returns the target cloud-volume entity.
    #[must_use]
    pub const fn volume(self) -> Entity {
        self.volume
    }

    /// Returns this light's role in the cloud volume.
    #[must_use]
    pub const fn kind(self) -> NimbusCloudLightKind {
        self.kind
    }

    /// Returns the explicit strength override, if present.
    #[must_use]
    pub const fn strength_override(self) -> Option<f32> {
        self.strength_override
    }

    /// Overrides Bevy illuminance with an explicit cloud-light multiplier.
    #[must_use]
    pub const fn with_strength(mut self, strength: f32) -> Self {
        self.strength_override = Some(strength);
        self
    }
}

/// Installs insertion-time validation for cloud-light relationships.
#[derive(Debug, Default)]
#[doc(hidden)]
pub struct NimbusLightingPlugin;

impl Plugin for NimbusLightingPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(validate_inserted_light);
    }
}

#[expect(clippy::needless_pass_by_value, reason = "Bevy observer parameter")]
fn validate_inserted_light(
    insert: On<Insert, NimbusCloudLight>,
    inserted: Query<&NimbusCloudLight>,
    lights: Query<&NimbusCloudLight>,
) {
    let Ok(binding) = inserted.get(insert.entity) else {
        return;
    };
    validate_light_bindings(binding.volume, lights.iter());
}

fn validate_light_bindings<'a>(
    volume: Entity,
    bindings: impl Iterator<Item = &'a NimbusCloudLight>,
) {
    let mut count = 0;
    let mut primary_count = 0;
    for binding in bindings.filter(|binding| binding.volume == volume) {
        count += 1;
        primary_count += usize::from(binding.kind == NimbusCloudLightKind::Primary);
        assert!(
            count <= MAX_DIRECTIONAL_LIGHTS,
            "Nimbus cloud volume {volume:?} exceeds {MAX_DIRECTIONAL_LIGHTS} directional lights",
        );
        assert!(
            primary_count <= 1,
            "Nimbus cloud volume {volume:?} has multiple primary directional lights",
        );
    }
}

/// Resolved, bounded lighting ready for the cloud uniform.
#[doc(hidden)]
pub struct ResolvedLighting {
    pub directional_lights: [DirectionalLightUniform; MAX_DIRECTIONAL_LIGHTS],
    pub directional_light_count: u32,
    pub ambient_strength: f32,
    pub ambient_color: Vec3,
}

/// Resolves ECS light entities affecting `volume`.
///
/// # Panics
/// Panics when a volume has more than [`MAX_DIRECTIONAL_LIGHTS`] lights or does
/// not have exactly one primary light.
#[must_use]
#[doc(hidden)]
pub fn resolve(
    volume: Entity,
    ambient: &NimbusAmbientLight,
    base_altitude: f32,
    thickness: f32,
    scene_lights: &Query<(
        Entity,
        &NimbusCloudLight,
        &DirectionalLight,
        &GlobalTransform,
        &InheritedVisibility,
    )>,
) -> ResolvedLighting {
    let base_altitude = if base_altitude.is_finite() {
        base_altitude
    } else {
        1500.0
    };
    let thickness = if thickness.is_finite() && thickness > 0.0 {
        thickness
    } else {
        1000.0
    };
    let mut sources: Vec<_> = scene_lights
        .iter()
        .filter(|(_, binding, _, _, inherited_visibility)| {
            binding.volume == volume
                && (binding.kind == NimbusCloudLightKind::Primary || inherited_visibility.get())
        })
        .collect();
    assert!(
        sources.len() <= MAX_DIRECTIONAL_LIGHTS,
        "Nimbus cloud volume {volume:?} has {} directional lights; maximum is {MAX_DIRECTIONAL_LIGHTS}",
        sources.len(),
    );
    sources.sort_unstable_by_key(|(entity, binding, _, _, _)| {
        (
            binding.kind != NimbusCloudLightKind::Primary,
            entity.to_bits(),
        )
    });
    assert!(
        sources.is_empty()
            || sources
                .iter()
                .filter(|source| source.1.kind == NimbusCloudLightKind::Primary)
                .count()
                == 1,
        "Nimbus cloud volume {volume:?} must have exactly one primary directional light",
    );

    let base_dip = horizon_dip(base_altitude);
    let top_dip = horizon_dip(base_altitude + thickness);
    let deck_dip = f32::midpoint(base_dip, top_dip);
    let mut directional_lights = [DirectionalLightUniform::default(); MAX_DIRECTIONAL_LIGHTS];
    for (index, (_, binding, light, transform, inherited_visibility)) in sources.iter().enumerate()
    {
        let direction_to_light: Vec3 = transform.back().into();
        let elevation = direction_to_light.y.clamp(-1.0, 1.0).asin();
        let horizon_visibility = deck_horizon_visibility(elevation, base_dip, top_dip);
        directional_lights[index] = DirectionalLightUniform {
            direction_to_light,
            strength: if inherited_visibility.get() {
                binding
                    .strength_override
                    .unwrap_or(light.illuminance / lux::RAW_SUNLIGHT)
                    .max(0.0)
                    * horizon_visibility
            } else {
                0.0
            },
            color: light.color.to_linear().to_vec3() * low_sun_tint(elevation, deck_dip),
        };
    }
    ResolvedLighting {
        directional_lights,
        directional_light_count: u32::try_from(sources.len())
            .expect("directional-light limit fits u32"),
        ambient_strength: ambient.strength.max(0.0),
        ambient_color: ambient.color.to_linear().to_vec3(),
    }
}

fn horizon_dip(altitude_meters: f32) -> f32 {
    let altitude = altitude_meters.max(0.0);
    (EARTH_RADIUS_METERS / (EARTH_RADIUS_METERS + altitude)).acos()
}

fn deck_horizon_visibility(elevation: f32, base_dip: f32, top_dip: f32) -> f32 {
    let half_softness = 0.5 * HORIZON_SOFTNESS_RADIANS;
    let fully_hidden = -top_dip - half_softness;
    let fully_visible = -base_dip + half_softness;
    let normalized = ((elevation - fully_hidden) / (fully_visible - fully_hidden)).clamp(0.0, 1.0);
    normalized * normalized * (3.0 - 2.0 * normalized)
}

fn low_sun_tint(elevation: f32, deck_dip: f32) -> Vec3 {
    let apparent_elevation = (elevation + deck_dip).clamp(0.0, std::f32::consts::FRAC_PI_2);
    let apparent_degrees = apparent_elevation.to_degrees();
    // Kasten-Young relative optical air mass remains finite at the apparent horizon.
    let air_mass =
        1.0 / (apparent_elevation.sin() + 0.505_72 * (apparent_degrees + 6.079_95).powf(-1.636_4));
    let excess_air_mass = (air_mass - 1.0).max(0.0);
    let optical_depth = RAYLEIGH_TINT_OPTICAL_DEPTH * excess_air_mass;
    let transmittance = Vec3::new(
        (-optical_depth.x).exp(),
        (-optical_depth.y).exp(),
        (-optical_depth.z).exp(),
    );
    transmittance / transmittance.max_element().max(f32::EPSILON)
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    reason = "these tests assert exact contract constants"
)]
mod tests {
    use super::{
        HORIZON_SOFTNESS_RADIANS, NimbusAmbientLight, NimbusCloudLight, NimbusCloudLightKind,
        NimbusLightingPlugin, deck_horizon_visibility, horizon_dip, low_sun_tint, resolve,
    };
    use bevy::prelude::*;

    #[test]
    fn primary_light_resolves_first_and_strength_is_clamped() {
        let mut world = World::new();
        let volume = world.spawn_empty().id();
        world.spawn((
            DirectionalLight::default(),
            GlobalTransform::IDENTITY,
            InheritedVisibility::VISIBLE,
            NimbusCloudLight::new(volume, NimbusCloudLightKind::Secondary),
        ));
        world.spawn((
            DirectionalLight::default(),
            GlobalTransform::from(Transform::from_rotation(Quat::from_rotation_x(
                -std::f32::consts::FRAC_PI_2,
            ))),
            InheritedVisibility::VISIBLE,
            NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary).with_strength(-1.0),
        ));
        let mut query_state = world.query::<(
            Entity,
            &NimbusCloudLight,
            &DirectionalLight,
            &GlobalTransform,
            &InheritedVisibility,
        )>();
        let resolved = resolve(
            volume,
            &NimbusAmbientLight::default(),
            1500.0,
            800.0,
            &query_state.query(&world),
        );

        assert_eq!(resolved.directional_light_count, 2);
        assert_eq!(resolved.directional_lights[0].strength, 0.0);
        assert!(resolved.directional_lights[0].direction_to_light.y > 0.99);
    }

    #[test]
    #[should_panic(expected = "multiple primary directional lights")]
    fn multiple_primary_lights_are_rejected_at_insertion() {
        let mut app = App::new();
        app.add_plugins(NimbusLightingPlugin);
        let volume = app.world_mut().spawn_empty().id();
        app.world_mut()
            .spawn(NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary));
        app.world_mut()
            .spawn(NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary));
    }

    #[test]
    #[should_panic(expected = "exceeds 4 directional lights")]
    fn fifth_light_is_rejected_at_insertion() {
        let mut app = App::new();
        app.add_plugins(NimbusLightingPlugin);
        let volume = app.world_mut().spawn_empty().id();
        app.world_mut()
            .spawn(NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary));
        for _ in 0..4 {
            app.world_mut().spawn(NimbusCloudLight::new(
                volume,
                NimbusCloudLightKind::Secondary,
            ));
        }
    }

    #[test]
    fn cloud_deck_primary_light_fades_below_ground_horizon() {
        let mut world = World::new();
        let volume = world.spawn_empty().id();
        let elevation = -1.4_f32.to_radians();
        let direction = Vec3::new(0.0, elevation.sin(), elevation.cos());
        world.spawn((
            DirectionalLight::default(),
            GlobalTransform::from(Transform::IDENTITY.looking_to(-direction, Vec3::Y)),
            InheritedVisibility::VISIBLE,
            NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary).with_strength(1.0),
        ));
        let mut query_state = world.query::<(
            Entity,
            &NimbusCloudLight,
            &DirectionalLight,
            &GlobalTransform,
            &InheritedVisibility,
        )>();
        let resolved = resolve(
            volume,
            &NimbusAmbientLight::default(),
            1500.0,
            800.0,
            &query_state.query(&world),
        );

        let light = resolved.directional_lights[0];
        assert!(light.strength > 0.0);
        assert!(light.strength < 1.0);
        assert!(light.color.x > light.color.y);
        assert!(light.color.y > light.color.z);
    }

    #[test]
    fn horizon_dip_matches_cloud_altitudes() {
        assert!((horizon_dip(1500.0).to_degrees() - 1.243).abs() < 0.002);
        assert!((horizon_dip(2300.0).to_degrees() - 1.539).abs() < 0.002);
    }

    #[test]
    fn deck_visibility_tracks_base_then_top_with_physical_softness() {
        let base_dip = horizon_dip(1500.0);
        let top_dip = horizon_dip(2300.0);
        let base_edge = deck_horizon_visibility(-base_dip, base_dip, top_dip);
        let top_edge = deck_horizon_visibility(-top_dip, base_dip, top_dip);

        assert!(base_edge > top_edge);
        assert!(base_edge < 1.0);
        assert!(top_edge > 0.0);
        assert_eq!(
            deck_horizon_visibility(-top_dip - HORIZON_SOFTNESS_RADIANS, base_dip, top_dip,),
            0.0
        );
        assert_eq!(deck_horizon_visibility(0.0, base_dip, top_dip), 1.0);
    }

    #[test]
    fn analytic_low_sun_tint_reduces_short_wavelengths() {
        let deck_dip = f32::midpoint(horizon_dip(1500.0), horizon_dip(2300.0));
        let tint = low_sun_tint(-deck_dip, deck_dip);

        assert!((tint.x - 1.0).abs() < f32::EPSILON);
        assert!(tint.x > tint.y);
        assert!(tint.y > tint.z);
        assert!(tint.z < 0.4);
    }

    #[test]
    fn far_below_cloud_horizon_primary_light_has_zero_strength() {
        let mut world = World::new();
        let volume = world.spawn_empty().id();
        world.spawn((
            DirectionalLight::default(),
            GlobalTransform::from(Transform::from_rotation(Quat::from_rotation_x(
                std::f32::consts::FRAC_PI_2,
            ))),
            InheritedVisibility::VISIBLE,
            NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary),
        ));
        let mut query_state = world.query::<(
            Entity,
            &NimbusCloudLight,
            &DirectionalLight,
            &GlobalTransform,
            &InheritedVisibility,
        )>();
        let resolved = resolve(
            volume,
            &NimbusAmbientLight::default(),
            1500.0,
            800.0,
            &query_state.query(&world),
        );

        assert_eq!(resolved.directional_light_count, 1);
        assert_eq!(resolved.directional_lights[0].strength, 0.0);
    }

    #[test]
    fn hidden_primary_light_is_retained_with_zero_strength() {
        let mut world = World::new();
        let volume = world.spawn_empty().id();
        world.spawn((
            DirectionalLight::default(),
            GlobalTransform::IDENTITY,
            Visibility::Hidden,
            InheritedVisibility::HIDDEN,
            NimbusCloudLight::new(volume, NimbusCloudLightKind::Primary),
        ));
        let mut query_state = world.query::<(
            Entity,
            &NimbusCloudLight,
            &DirectionalLight,
            &GlobalTransform,
            &InheritedVisibility,
        )>();
        let resolved = resolve(
            volume,
            &NimbusAmbientLight::default(),
            1500.0,
            800.0,
            &query_state.query(&world),
        );

        assert_eq!(resolved.directional_light_count, 1);
        assert_eq!(resolved.directional_lights[0].strength, 0.0);
    }
}
