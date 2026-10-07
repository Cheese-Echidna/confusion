//! Semantic native feature keys include evaluated inputs and upstream shape keys.
#[cfg(all(feature = "solver", feature = "kernel"))]
pub(crate) fn feature_keys(
    edges: &[crate::kernel::bridge::ffi::ProfileEdge],
    steps: &[crate::kernel::bridge::ffi::ModelStep],
    creates: &[crate::kernel::bridge::ffi::CreateStep],
    edits: &[crate::kernel::bridge::ffi::ModifyStep],
) -> Result<Vec<String>, String> {
    use super::cache::key;
    let profile = |start: u32, count: u32| {
        edges
            .get(start as usize..(start as usize).checked_add(count as usize)?)
            .map(|edges| {
                edges
                    .iter()
                    .map(|e| (e.wire, e.sx, e.sy, e.ex, e.ey, e.cx, e.cy, e.sweep))
                    .collect::<Vec<_>>()
            })
    };
    let mut keys: Vec<String> = Vec::new();
    for step in steps {
        let dependency = |index: i32| -> Result<Option<&str>, String> {
            if index == -1 {
                Ok(None)
            } else {
                keys.get(index as usize)
                    .map(|k| Some(k.as_str()))
                    .ok_or("Invalid cache dependency".into())
            }
        };
        keys.push(key(&(
            (
                step.edge_start,
                step.edge_count,
                step.depth,
                step.operation,
                step.target,
                step.support,
                step.producer,
                step.role,
            ),
            profile(step.edge_start, step.edge_count).ok_or("Invalid profile range")?,
            dependency(step.target)?,
            dependency(step.support)?,
            dependency(step.producer)?,
        ))?);
    }
    for step in creates {
        let dependency = |index: i32| -> Result<Option<&str>, String> {
            if index == -1 {
                Ok(None)
            } else {
                keys.get(index as usize)
                    .map(|k| Some(k.as_str()))
                    .ok_or("Invalid cache dependency".into())
            }
        };
        keys.push(key(&(
            (
                step.kind,
                step.edge_start,
                step.edge_count,
                step.second_start,
                step.second_count,
                step.target,
                step.second_target,
                &step.values,
            ),
            profile(step.edge_start, step.edge_count).ok_or("Invalid profile range")?,
            profile(step.second_start, step.second_count).ok_or("Invalid profile range")?,
            dependency(step.target)?,
            dependency(step.second_target)?,
        ))?);
    }
    for step in edits {
        // Face ordinals depend on the complete preceding model, so edits use
        // conservative prefix invalidation rather than target-only invalidation.
        keys.push(key(&(
            (
                step.kind,
                step.target,
                step.tool,
                step.face,
                step.a,
                step.b,
                step.c,
                step.d,
                step.copy,
                step.mode,
            ),
            &keys,
        ))?);
    }
    Ok(keys)
}
