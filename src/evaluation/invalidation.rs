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
                    .map(|e| {
                        (
                            (e.wire, e.sx, e.sy, e.ex, e.ey, e.cx, e.cy, e.sweep),
                            (
                                e.kind,
                                e.from,
                                e.to,
                                &e.identity,
                                e.poles.iter().map(|p| (p.x, p.y)).collect::<Vec<_>>(),
                            ),
                        )
                    })
                    .collect::<Vec<_>>()
            })
    };
    let mut keys = vec![String::new(); steps.len() + creates.len() + edits.len()];
    let mut bodies = vec![String::new(); steps.len() + creates.len()];
    let mut events = Vec::new();
    for (i, s) in steps.iter().enumerate() {
        events.push((s.sequence, 0, i));
    }
    for (i, s) in creates.iter().enumerate() {
        events.push((s.sequence, 1, i));
    }
    for (i, s) in edits.iter().enumerate() {
        events.push((s.sequence, 2, i));
    }
    events.sort_by_key(|e| e.0);
    for (_, kind, i) in events {
        let dependency = |index: i32| -> Result<Option<&str>, String> {
            if index == -1 {
                Ok(None)
            } else {
                bodies
                    .get(index as usize)
                    .filter(|k| !k.is_empty())
                    .map(|k| Some(k.as_str()))
                    .ok_or("Invalid cache dependency".into())
            }
        };
        let (slot, value) = match kind {
            0 => {
                let s = &steps[i];
                (
                    i,
                    key(&(
                        (
                            &s.identity,
                            s.sequence,
                            &s.reference,
                            s.plane,
                            s.depth,
                            s.operation,
                            s.target,
                            s.support,
                            s.producer,
                            s.role,
                        ),
                        profile(s.edge_start, s.edge_count).ok_or("Invalid profile range")?,
                        dependency(s.target)?,
                        dependency(s.support)?,
                        dependency(s.producer)?,
                    ))?,
                )
            }
            1 => {
                let s = &creates[i];
                (
                    steps.len() + i,
                    key(&(
                        (
                            &s.identity,
                            s.sequence,
                            s.kind,
                            s.target,
                            s.second_target,
                            &s.values,
                        ),
                        profile(s.edge_start, s.edge_count).ok_or("Invalid profile range")?,
                        profile(s.second_start, s.second_count).ok_or("Invalid profile range")?,
                        dependency(s.target)?,
                        dependency(s.second_target)?,
                    ))?,
                )
            }
            _ => {
                let s = &edits[i];
                (
                    steps.len() + creates.len() + i,
                    key(&(
                        (
                            s.kind, s.target, s.tool, s.face, s.a, s.b, s.c, s.d, s.copy, s.mode,
                        ),
                        (
                            &s.identity,
                            s.sequence,
                            &s.face_reference,
                            &s.tool_reference,
                            s.edge_points
                                .iter()
                                .map(|p| (p.x, p.y, p.z))
                                .collect::<Vec<_>>(),
                        ),
                        &bodies,
                    ))?,
                )
            }
        };
        keys[slot] = value.clone();
        if kind < 2 {
            bodies[slot] = value;
        } else {
            bodies[edits[i].target as usize] = value;
        }
    }
    Ok(keys)
}
