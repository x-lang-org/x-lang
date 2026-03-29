// 两数相加
// https://leetcode.cn/problems/add-two-numbers/

needs stdio

// Definition for singly-linked list.
type ListNode = struct {
    val: int
    next: ?ListNode
}

can add_two_numbers(l1: ?ListNode, l2: ?ListNode) -> ?ListNode {
    given dummy: ListNode = ListNode { val = 0, next = null }
    var current = &dummy
    carry = 0

    var p1 = l1
    var p2 = l2

    while p1 is not null or p2 is not null or carry > 0 {
        sum = carry
        if p1 is not null {
            sum += p1.val
            p1 = p1.next
        }
        if p2 is not null {
            sum += p2.val
            p2 = p2.next
        }
        carry = sum / 10
        current.next = Some ListNode { val = sum % 10, next = null }
        current = current.next as &ListNode
    }

    return dummy.next
}

// Helper to create linked list from array
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

// Helper to print linked list
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
    // Read input: two arrays
    n1 = 0
    _ = scanf("%d", &n1)
    given a1: []int = [] with cap n1
    for i in 0..n1-1 {
        x = 0
        _ = scanf("%d", &x)
        a1 push x
    }

    n2 = 0
    _ = scanf("%d", &n2)
    given a2: []int = [] with cap n2
    for i in 0..n2-1 {
        x = 0
        _ = scanf("%d", &x)
        a2 push x
    }

    l1 = create_list(a1)
    l2 = create_list(a2)
    result = add_two_numbers(l1, l2)
    print_list(result)
}
