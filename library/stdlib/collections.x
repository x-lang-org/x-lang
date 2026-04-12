module std.collections

import std.prelude;
import std.types;

/// 哈希集合 - 基于 Map 实现
export record HashSet<T> where T: Eq {
    /// 使用 Map 的键来存储集合，值只是占位符
    data: Map<T, unit>,
}

/// 创建空的哈希集合
export function empty_set<T> where T: Eq -> HashSet<T> {
    HashSet { data: std.types.empty() }
}

/// 创建哈希集合并从数组初始化
export function set_from_array<T> where T: Eq(items: [T]) -> HashSet<T> {
    let mut set = empty_set();
    for item in items {
        set.insert(item);
    }
    set
}

/// 获取集合大小
export function len<T> where T: Eq(self: HashSet<T>) -> Int {
    self.data.len()
}

/// 判断集合是否为空
export function is_empty<T> where T: Eq(self: HashSet<T>) -> Bool {
    self.data.is_empty()
}

/// 插入元素
export function insert<T> where T: Eq(self: &mut HashSet<T>, value: T) -> Bool {
    // 返回 true 如果元素已经存在（被替换）
    match self.data.insert(value, unit) {
        Some(_) => true,
        None => false,
    }
}

/// 移除元素
export function remove<T> where T: Eq(self: &mut HashSet<T>, value: T) -> Option<unit> {
    self.data.remove(value)
}

/// 检查集合是否包含元素
export function contains<T> where T: Eq(self: HashSet<T>, value: T) -> Bool {
    self.data.contains_key(value)
}

/// 清空集合
export function clear<T> where T: Eq(self: &mut HashSet<T>) -> unit {
    while not self.is_empty() {
        // 这不是最高效的实现，但对于简单实现足够了
        for (k, _) in self.data.entries {
            self.data.remove(k);
            break;
        }
    }
}

/// 迭代器适配器：遍历集合中的元素
export function for_each<T> where T: Eq(self: HashSet<T>, f: function(T) -> unit) -> unit {
    for (k, _) in self.data.entries {
        f(k);
    }
}

/// 集合转换为列表
export function to_list<T> where T: Eq(self: HashSet<T>) -> List<T> {
    let mut result = std.types.empty();
    self.for_each(|item| result.push(item));
    result
}

/// 栈 - 后进先出 (LIFO)
export record Stack<T> {
    items: List<T>,
}

/// 创建空栈
export function empty_stack<T>() -> Stack<T> {
    Stack { items: std.types.empty() }
}

/// 创建栈从数组
export function stack_from_array<T>(items: [T]) -> Stack<T> {
    Stack { items: std.types.from_array(items) }
}

/// 获取栈大小
export function len<T>(self: Stack<T>) -> Int {
    self.items.len()
}

/// 判断栈是否为空
export function is_empty<T>(self: Stack<T>) -> Bool {
    self.items.is_empty()
}

/// 压入元素
export fn push<T>(self: &mut Stack<T>, value: T) -> unit {
    self.items.push(value);
}

/// 弹出元素
export fn pop<T>(self: &mut Stack<T>) -> Option<T> {
    self.items.pop()
}

/// 查看栈顶元素（不弹出）
export fn peek<T>(self: Stack<T>) -> Option<T> {
    self.items.last()
}

/// 清空栈
export fn clear<T>(self: &mut Stack<T>) -> unit {
    self.items.clear();
}

/// 转换为列表
export fn to_list<T>(self: Stack<T>) -> List<T> {
    self.items
}

/// 队列 - 先进先出 (FIFO)
/// 使用两个栈实现，摊还 O(1) 复杂度
export record Queue<T> {
    /// 入队栈
    in_stack: Stack<T>,
    /// 出队栈
    out_stack: Stack<T>,
}

/// 创建空队列
export fn empty_queue<T>() -> Queue<T> {
    Queue {
        in_stack: empty_stack(),
        out_stack: empty_stack(),
    }
}

/// 私有方法：将入栈移动到出栈
private fn transfer<T>(self: &mut Queue<T>) -> unit {
    while not self.in_stack.is_empty() {
        self.out_stack.push(self.in_stack.pop().unwrap());
    }
}

/// 获取队列大小
export fn len<T>(self: Queue<T>) -> Int {
    self.in_stack.len() + self.out_stack.len()
}

/// 判断队列是否为空
export fn is_empty<T>(self: Queue<T>) -> Bool {
    self.len() == 0
}

/// 入队
export fn enqueue<T>(self: &mut Queue<T>, value: T) -> unit {
    self.in_stack.push(value);
}

/// 出队
export fn dequeue<T>(self: &mut Queue<T>) -> Option<T> {
    when self.out_stack.is_empty() {
        self.transfer();
    }
    self.out_stack.pop()
}

/// 查看队首元素
export fn peek<T>(self: &mut Queue<T>) -> Option<T> {
    when self.out_stack.is_empty() {
        self.transfer();
    }
    self.out_stack.peek()
}

/// 清空队列
export fn clear<T>(self: &mut Queue<T>) -> unit {
    self.in_stack.clear();
    self.out_stack.clear();
}

/// 双向链表节点
export record Node<T> {
    value: T,
    prev: Option<*Node<T>>,
    next: Option<*Node<T>>,
}

/// 双向链表
export record LinkedList<T> {
    head: Option<*Node<T>>,
    tail: Option<*Node<T>>,
    length: Int,
}

/// 创建空链表
export fn empty_linked_list<T>() -> LinkedList<T> {
    LinkedList {
        head: None,
        tail: None,
        length: 0,
    }
}

/// 获取链表长度
export fn len<T>(self: LinkedList<T>) -> Int {
    self.length
}

/// 判断链表是否为空
export fn is_empty<T>(self: LinkedList<T>) -> Bool {
    self.length == 0
}

/// 追加到末尾
/// Note: 需要 unsafe 操作指针
export fn push_back<T>(self: &mut LinkedList<T>, value: T) -> unit {
    // 在实际实现中，我们需要内存分配器
    // 现在这只是一个逻辑结构，实际分配由运行时处理
    // 对于 X 标准库，这里展示接口设计
    unsafe {
        let node: *Node<T> = alloc_one.<Node<T>>();
        (*node) = Node {
            value: value,
            prev: self.tail,
            next: None,
        };
        match self.tail {
            None => {
                self.head = Some(node);
                self.tail = Some(node);
            }
            Some(tail) => {
                (*tail).next = Some(node);
                self.tail = Some(node);
            }
        }
        self.length = self.length + 1;
    }
}

/// 二叉搜索树（简单实现）
export record BSTNode<K, V> where K: Ord {
    key: K,
    value: V,
    left: Option<*BSTNode<K, V>>,
    right: Option<*BSTNode<K, V>>,
}

export record BinarySearchTree<K, V> where K: Ord {
    root: Option<*BSTNode<K, V>>,
    size: Int,
}

/// 创建空二叉搜索树
export fn empty_bst<K, V> where K: Ord -> BinarySearchTree<K, V> {
    BinarySearchTree {
        root: None,
        size: 0,
    }
}

/// 获取大小
export fn size<K, V> where K: Ord(self: BinarySearchTree<K, V>) -> Int {
    self.size
}

/// 二叉堆（最小堆）
export record MinHeap<T> where T: Ord {
    data: List<T>,
}

/// 创建空最小堆
export fn empty_min_heap<T> where T: Ord -> MinHeap<T> {
    MinHeap { data: std.types.empty() }
}

/// 获取堆大小
export fn len<T> where T: Ord(self: MinHeap<T>) -> Int {
    self.data.len()
}

/// 判断堆是否为空
export fn is_empty<T> where T: Ord(self: MinHeap<T>) -> Bool {
    self.data.is_empty()
}

/// 上浮操作
private fn sift_up<T> where T: Ord(self: &mut MinHeap<T>, mut index: Int) -> unit {
    while index > 0 {
        let parent = (index - 1) / 2;
        when self.data.get(index).unwrap() < self.data.get(parent).unwrap() {
            // 交换
            let temp = self.data.get(index).unwrap();
            self.data.replace(index, self.data.get(parent).unwrap());
            self.data.replace(parent, temp);
            index = parent;
        } else {
            break;
        }
    }
}

/// 下沉操作
private fn sift_down<T> where T: Ord(self: &mut MinHeap<T>, mut index: Int) -> unit {
    let n = self.len();
    while 2 * index + 1 < n {
        let left = 2 * index + 1;
        let right = 2 * index + 2;
        let mut smallest = index;
        when left < n and self.data.get(left).unwrap() < self.data.get(smallest).unwrap() {
            smallest = left;
        }
        when right < n and self.data.get(right).unwrap() < self.data.get(smallest).unwrap() {
            smallest = right;
        }
        when smallest != index {
            // 交换
            let temp = self.data.get(index).unwrap();
            self.data.replace(index, self.data.get(smallest).unwrap());
            self.data.replace(smallest, temp);
            index = smallest;
        } else {
            break;
        }
    }
}

/// 插入元素到堆
export fn push<T> where T: Ord(self: &mut MinHeap<T>, value: T) -> unit {
    self.data.push(value);
    self.sift_up(self.len() - 1);
}

/// 弹出最小元素
export fn pop<T> where T: Ord(self: &mut MinHeap<T>) -> Option<T> {
    when self.is_empty() {
        None
    } else {
        let result = self.data.get(0).unwrap();
        let last = self.data.pop().unwrap();
        when not self.is_empty() {
            self.data.replace(0, last);
            self.sift_down(0);
        }
        Some(result)
    }
}

/// 查看最小元素
export fn peek_min<T> where T: Ord(self: MinHeap<T>) -> Option<T> {
    when self.is_empty() {
        None
    } else {
        Some(self.data.get(0).unwrap())
    }
}

/// 最大堆（基于最小堆反转比较）
export record MaxHeap<T> where T: Ord {
    data: MinHeap<Reverse<T>>,
}

/// 包装器反转比较顺序
export record Reverse<T> where T: Ord {
    value: T,
}

/// 创建空最大堆
export fn empty_max_heap<T> where T: Ord -> MaxHeap<T> {
    MaxHeap { data: empty_min_heap() }
}

/// 获取堆大小
export fn len<T> where T: Ord(self: MaxHeap<T>) -> Int {
    self.data.len()
}

/// 判断堆是否为空
export fn is_empty<T> where T: Ord(self: MaxHeap<T>) -> Bool {
    self.data.is_empty()
}

/// 插入元素
export fn push<T> where T: Ord(self: &mut MaxHeap<T>, value: T) -> unit {
    self.data.push(Reverse { value: value });
}

/// 弹出最大元素
export fn pop<T> where T: Ord(self: &mut MaxHeap<T>) -> Option<T> {
    match self.data.pop() {
        None => None,
        Some(reversed) => Some(reversed.value),
    }
}

/// 查看最大元素
export fn peek_max<T> where T: Ord(self: MaxHeap<T>) -> Option<T> {
    match self.data.peek_min() {
        None => None,
        Some(reversed) => Some(reversed.value),
    }
}
