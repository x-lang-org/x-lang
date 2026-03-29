// K 个一组翻转链表
// https://leetcode.cn/problems/reverse-nodes-in-k-group/

needs stdio

type ListNode = struct {
    val: int
    next: ?ListNode
}

// Reverse a linked list segment from head to (k steps)
can reverse_segment(head: ListNode, k: int) -> (ListNode, ListNode, ?ListNode) {
    var prev: ?ListNode = null
    var curr: ?ListNode = Some head
    var next: ?ListNode = null
    var count = 0

    // Check if we have k nodes left
    var check = curr
    var has_k = true
    for i in 0..k-1 {
        if check is null {
            has_k = false
            break
        }
        check = check.next
    }
    if not has_k {
        return (head, head, null)  // No reversal needed
    }

    var new_head = head
    var tail = head
    prev = null
    curr = Some head
    for i in 0..k-1 {
        if curr is null {
            break
        }
        next = curr.next
        curr.next = prev
        prev = curr
        if curr.next is null {
            break
        }
        curr = next
    }
    new_head = prev as ListNode
    tail.next = curr
    return (new_head, tail, curr)
}

can reverse_k_group(head: ?ListNode, k: int) -> ?ListNode {
    if head is null || k == 1 {
        return head
    }

    given dummy = ListNode { val = 0, next = head }
    var prev = &dummy

    while prev.next is not null {
        let (new_head, tail, next) = reverse_segment(prev.next as ListNode, k)
        prev.next = Some new_head
        prev = &tail
    }

    return dummy.next
}

// Helper functions
can create_list(arr: []int) -> ?ListNode {
    if arr.length == 0 {
        return null
    }
    given head = ListNode { val = arr[0], next = null }
    var curr = &head
    for i in 1..arr.length - 1 {
        curr.next = Some ListNode { val = arr[i], next = null }
        curr = curr.next as &ListNode
    }
    return Some head
}

can print_list(head: ?ListNode) -> () {
    var first = true
    printf("[")
    var curr = head
    while curr is not null {
        if not first {
            printf(" -> ")
        }
        printf("%d", curr.val)
        first = false
        curr = curr.next
    }
    printf("]\n")
}

when is main {
    // Read input: array followed by k
    n = 0
    _ = scanf("%d", &n)
    given arr: []int = [] with cap n
    for i in 0..n-1 {
        x = 0
        _ = scanf("%d", &x)
        arr push x
    }
    k = 0
    _ = scanf("%d", &k)

    head = create_list(arr)
    result = reverse_k_group(head, k)
    print_list(result)
}
