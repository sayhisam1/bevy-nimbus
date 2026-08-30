//! Shared cloud settings and GPU ABI for Nimbus.

mod settings;
mod uniform;

#[doc(hidden)]
pub use settings::CloudProfile;
pub use settings::{
    CloudPreset, CloudSettings, NimbusAmbientLight, NimbusCloudView, NimbusCloudVolume,
    NimbusSampling,
};
#[doc(hidden)]
pub use uniform::{
    CloudsUniform, CloudsUniformBuffer, DirectionalLightUniform, MAX_DIRECTIONAL_LIGHTS,
};
