use std::collections::HashSet;

fn connections_helper(
    arr: &mut [Option<u32>],
    index: usize,
    equivalents: impl Fn(&[u32]) -> Vec<Vec<u32>> + Clone,
    equivalent_set: &mut HashSet<Vec<u32>>,
) -> Vec<Vec<u32>> {
    if index == arr.len() {
        let connect = arr.iter().map(|n| n.unwrap()).collect::<Vec<_>>();
        for equiv in equivalents(&connect).into_iter().collect::<HashSet<_>>() {
            if !equivalent_set.insert(equiv) {
                return vec![];
            }
        }
        vec![connect]
    } else if arr[index].is_some() {
        // already connected
        connections_helper(arr, index + 1, equivalents, equivalent_set)
    } else {
        // connection time!
        let mut out = vec![];
        
        for i in (index + 1)..arr.len() {
            if arr[i].is_none() {
                arr[index] = Some(i as u32);
                arr[i] = Some(index as u32);
                out.extend(connections_helper(arr, index + 1, equivalents.clone(), equivalent_set));
                arr[index] = None;
                arr[i] = None;
            }
        }

        out
    }
}

pub fn equivalent_rotation_180(connection: &[u32]) -> Vec<Vec<u32>> {
    let len = connection.len();
    let split = connection.len() / 2;

    vec![
        connection.to_vec(),
        (0..connection.len()).map(|i| (connection[(i + split) % len] + split as u32) % len as u32).collect(),
    ]
}

/// Returns a list of all the ways to connect `num_ports` ports 2 at a time.
/// 
/// A way to connect is a list `a` where `a[i] == j` iff port `i` is connected to port `j`.
pub fn connections(num_ports: usize, equivalents: impl Fn(&[u32]) -> Vec<Vec<u32>> + Clone) -> Vec<Vec<u32>> {
    let mut arr = vec![None; num_ports];
    let mut set = HashSet::new();
    connections_helper(&mut arr, 0, equivalents, &mut set)
}