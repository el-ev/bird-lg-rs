use crate::models::{DiffOp, Protocol};

pub fn apply_protocol_diff(existing: &[Protocol], diff: &[DiffOp]) -> Vec<Protocol> {
    let mut next = Vec::new();
    let mut old_idx = 0;

    for op in diff {
        match op {
            DiffOp::Equal { c: count } => {
                if old_idx + count <= existing.len() {
                    next.extend_from_slice(&existing[old_idx..old_idx + count]);
                    old_idx += count;
                }
            }
            DiffOp::Insert { i: items } => next.extend(items.iter().cloned()),
            DiffOp::Delete { c: count } => old_idx += count,
            DiffOp::Replace { i: items } => {
                next.extend(items.iter().cloned());
                old_idx += items.len();
            }
        }
    }

    next
}

#[cfg(test)]
mod tests {
    use super::apply_protocol_diff;
    use crate::models::{DiffOp, Protocol};

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
    fn applies_insert_delete_and_equal_ops() {
        let current = vec![protocol("a", "old-a"), protocol("b", "old-b")];
        let diff = vec![
            DiffOp::Equal { c: 1 },
            DiffOp::Delete { c: 1 },
            DiffOp::Insert {
                i: vec![protocol("c", "new-c")],
            },
        ];

        let updated = apply_protocol_diff(&current, &diff);
        assert_eq!(
            updated,
            vec![protocol("a", "old-a"), protocol("c", "new-c")]
        );
    }
}
