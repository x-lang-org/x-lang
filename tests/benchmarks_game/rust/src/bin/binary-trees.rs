enum TreeOption {
    Some(Box<TreeNode>),
    None,
}

struct TreeNode {
    left: TreeOption,
    right: TreeOption,
}

fn bottom_up_tree(depth: i32) -> TreeNode {
    if depth > 0 {
        TreeNode {
            left: TreeOption::Some(Box::new(bottom_up_tree(depth - 1))),
            right: TreeOption::Some(Box::new(bottom_up_tree(depth - 1))),
        }
    } else {
        TreeNode {
            left: TreeOption::None,
            right: TreeOption::None,
        }
    }
}

fn item_check(node: &TreeNode) -> i32 {
    match (&node.left, &node.right) {
        (TreeOption::Some(left), TreeOption::Some(right)) => {
            1 + item_check(left) + item_check(right)
        }
        _ => 1,
    }
}

fn main() {
    let min_depth = 4;
    let max_depth = 10;
    let stretch_depth = max_depth + 1;
    
    let stretch_tree = bottom_up_tree(stretch_depth);
    println!("stretch tree of depth {} check: {}", stretch_depth, item_check(&stretch_tree));
    
    let long_lived_tree = bottom_up_tree(max_depth);
    
    let mut depth = min_depth;
    while depth <= max_depth {
        let iterations = 1 << (max_depth - depth + min_depth);
        let mut check = 0;
        for _ in 0..iterations {
            let t = bottom_up_tree(depth);
            check += item_check(&t);
        }
        println!("{} trees of depth {} check: {}", iterations, depth, check);
        depth += 2;
    }
    
    println!("long lived tree of depth {} check: {}", max_depth, item_check(&long_lived_tree));
}
