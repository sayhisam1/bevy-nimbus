# Nimbus examples

Each example demonstrates one Nimbus API feature and runs on native platforms or WebGPU.

| Example | Feature |
|---|---|
| `basic_clouds` | Minimal cloud volume, view, and primary light |
| `cloud_settings` | Weather presets and `CloudSettings` |
| `sampling_quality` | `NimbusSampling` quality controls |
| `cloud_lighting` | Ambient, primary, and secondary cloud lighting |
| `cloud_shadows` | Projecting `NimbusCloudShadowMap` onto scene geometry |

Run an example locally:

```sh
cargo run --release --example basic_clouds
```

`cloud_settings` and `sampling_quality` use Space to cycle their settings.
