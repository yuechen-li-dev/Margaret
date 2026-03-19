# Margaret architecture

## M0 intent

M0 establishes crate boundaries and a compile-clean scaffold for future rendering work. This pass deliberately avoids real rendering algorithms, real acceleration structures, scene file parsing, and backend-specific execution logic.

## Crate layout

- `margaret-core`: shared renderer-facing data types and small contracts. This crate owns scene description, camera, color, math, ray, material, light, and output metadata skeletons.
- `margaret-cpu`: CPU backend scaffold. It currently reports placeholder render metadata for a scene and does not perform rendering.
- `margaret-vk`: Vulkan backend scaffold. It exists to quarantine future Vulkan work behind a backend crate without leaking Vulkan concerns into higher-level crates.
- `margaret-image`: simple owned image helpers for placeholder output buffers.
- `margaret-cli`: binary harness that builds a hardcoded placeholder scene and prints placeholder render metadata.
- `margaret-testutil`: shared test helpers for stable sample scenes and image sizes.

## Boundary rules

- Keep shared renderer concepts in `margaret-core`.
- Keep backend-specific work inside `margaret-cpu` and `margaret-vk`.
- Keep `margaret-vk` as scaffold-only until real backend work begins.
- Keep `margaret-cli` thin; it should compose crates rather than own renderer logic.
- Keep `margaret-image` focused on owned image buffer helpers rather than scene or backend policy.

## Immediate follow-on direction

M1 should begin filling in CPU-side render flow with explicit, testable steps while preserving the crate boundaries established here. Vulkan work should remain quarantined until there is a concrete backend need.
