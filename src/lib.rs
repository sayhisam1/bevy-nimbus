//! True-volumetric Nubis cloud rendering for Bevy 0.19.
//!
//! Add [`NimbusPlugin`], spawn one [`NimbusCloudVolume`], then connect a camera
//! with [`NimbusCloudView`] and a directional light with [`NimbusCloudLight`].
//! See the `basic_clouds` example for a minimal scene.
#![warn(missing_docs)]

pub use nimbus_render::{
    CloudPreset, CloudSettings, NimbusAmbientLight, NimbusCloudLight, NimbusCloudLightKind,
    NimbusCloudShadowMap, NimbusCloudShadowSource, NimbusCloudView, NimbusCloudVolume,
    NimbusPlugin, NimbusSampling,
};
