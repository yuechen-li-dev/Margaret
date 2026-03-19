# Margaret milestones

## M0 - workspace scaffold

Status: implemented in this pass.

Goals:
- Establish the Rust workspace and crate boundaries.
- Add minimal compile-clean library and binary entrypoints.
- Define small foundational core types for scenes, math, rays, colors, materials, lights, cameras, and output metadata.
- Add a CLI scaffold that builds a hardcoded placeholder scene and reports placeholder render metadata.

Non-goals:
- No real rendering.
- No real geometry intersection.
- No BVH traversal.
- No Vulkan renderer implementation.
- No scene file parsing or MaterialX support.
- No plugin or FFI layer.

## M1 - first CPU render path

Planned direction:
- Add a minimal CPU render loop using the M0 scene and image abstractions.
- Keep the implementation explicit and testable.
- Continue avoiding speculative abstraction until concrete duplication appears.
- Preserve backend quarantine so Vulkan remains optional and scaffolded.
