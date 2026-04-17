use common::models::{DiffOp, Protocol};

pub fn calculate_diff(old: &[Protocol], new: &[Protocol]) -> Vec<DiffOp> {
    let lcs_matrix = lcs(old, new);
    let mut diff = Vec::new();
    let mut i = old.len();
    let mut j = new.len();

    let mut path = Vec::new();

    while i > 0 && j > 0 {
        if old[i - 1].name == new[j - 1].name {
            path.push((i - 1, j - 1));
            i -= 1;
            j -= 1;
        } else if lcs_matrix[i - 1][j] > lcs_matrix[i][j - 1] {
            i -= 1;
        } else {
            j -= 1;
        }
    }
    path.reverse();

    let mut old_idx = 0;
    let mut new_idx = 0;

    for (match_old, match_new) in path {
        if match_old > old_idx {
            diff.push(DiffOp::Delete {
                c: match_old - old_idx,
            });
        }

        if match_new > new_idx {
            diff.push(DiffOp::Insert {
                i: new[new_idx..match_new].to_vec(),
            });
        }

        if old[match_old] == new[match_new] {
            if let Some(DiffOp::Equal { c: count }) = diff.last_mut() {
                *count += 1;
            } else {
                diff.push(DiffOp::Equal { c: 1 });
            }
        } else {
            diff.push(DiffOp::Replace {
                i: vec![new[match_new].clone()],
            });
        }

        old_idx = match_old + 1;
        new_idx = match_new + 1;
    }

    if old_idx < old.len() {
        diff.push(DiffOp::Delete {
            c: old.len() - old_idx,
        });
    }

    if new_idx < new.len() {
        diff.push(DiffOp::Insert {
            i: new[new_idx..].to_vec(),
        });
    }

    opt_fold(diff)
}

fn lcs(old: &[Protocol], new: &[Protocol]) -> Vec<Vec<usize>> {
    let m = old.len();
    let n = new.len();
    let mut c = vec![vec![0; n + 1]; m + 1];

    for i in 1..=m {
        for j in 1..=n {
            if old[i - 1].name == new[j - 1].name {
                c[i][j] = c[i - 1][j - 1] + 1;
            } else {
                c[i][j] = c[i - 1][j].max(c[i][j - 1]);
            }
        }
    }
    c
}

fn opt_fold(diff: Vec<DiffOp>) -> Vec<DiffOp> {
    let mut result = Vec::new();

    for op in diff {
        match op {
            DiffOp::Replace { i: items } => {
                if let Some(DiffOp::Replace { i: last }) = result.last_mut() {
                    last.extend(items);
                } else {
                    result.push(DiffOp::Replace { i: items });
                }
            }
            DiffOp::Insert { i: items } => {
                if let Some(DiffOp::Insert { i: last }) = result.last_mut() {
                    last.extend(items);
                } else {
                    result.push(DiffOp::Insert { i: items });
                }
            }
            DiffOp::Delete { c: count } => {
                if let Some(DiffOp::Delete { c: last_count }) = result.last_mut() {
                    *last_count += count;
                } else {
                    result.push(DiffOp::Delete { c: count });
                }
            }
            DiffOp::Equal { c: count } => {
                if let Some(DiffOp::Equal { c: last_count }) = result.last_mut() {
                    *last_count += count;
                } else {
                    result.push(DiffOp::Equal { c: count });
                }
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use common::{diff::apply_protocol_diff, models::Protocol};

    use super::calculate_diff;

    fn protocol(name: &str, info: &str) -> Protocol {
        Protocol {
            name: name.to_string(),
            proto: "BGP".to_string(),
            table: "master4".to_string(),
            state: "up".to_string(),
            since: "2026-04-17".to_string(),
            info: info.to_string(),
        }
    }

    #[test]
    fn diff_round_trips_protocol_lists() {
        let old = vec![
            protocol("peer-a", "established"),
            protocol("peer-b", "active"),
        ];
        let new = vec![
            protocol("peer-a", "established"),
            protocol("peer-c", "idle"),
        ];

        let diff = calculate_diff(&old, &new);
        let applied = apply_protocol_diff(&old, &diff);

        assert_eq!(applied, new);
    }
}
