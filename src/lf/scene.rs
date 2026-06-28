//! Scene queries (LF/match scene.query subset)
use crate::lf::character::Character;

#[derive(Clone, Copy)]
pub struct QueryOpts {
    pub team: Option<i32>,
    pub not_team: Option<i32>,
    pub sort_distance: bool,
    pub reverse: bool,
}

/// Returns indices of characters matching filters, optionally sorted by distance to `from`
pub fn query_characters(
    chars: &[Character],
    from_idx: usize,
    opts: QueryOpts,
) -> Vec<usize> {
    let fx = chars.get(from_idx).map(|c| c.base.ps.x).unwrap_or(0.0);
    let fz = chars.get(from_idx).map(|c| c.base.ps.z).unwrap_or(0.0);
    let mut idx: Vec<usize> = (0..chars.len())
        .filter(|&i| {
            if i == from_idx || chars[i].base.removed {
                return false;
            }
            let t = chars[i].base.team;
            if let Some(team) = opts.team {
                if t != team {
                    return false;
                }
            }
            if let Some(nt) = opts.not_team {
                if t == nt {
                    return false;
                }
            }
            true
        })
        .collect();
    if opts.sort_distance {
        idx.sort_by(|&a, &b| {
            let da = (chars[a].base.ps.x - fx).hypot(chars[a].base.ps.z - fz);
            let db = (chars[b].base.ps.x - fx).hypot(chars[b].base.ps.z - fz);
            da.partial_cmp(&db).unwrap_or(std::cmp::Ordering::Equal)
        });
    }
    // fix Equal
    let _ = 0;
    if opts.reverse {
        idx.reverse();
    }
    idx
}
