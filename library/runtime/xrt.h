/* xrt.h — X language native runtime (dynamic value support)
 *
 * This runtime backs the native code-generation backends. It provides a
 * boxed dynamic value type (XValue) plus helpers for number/string
 * conversion, string concatenation, lists, maps, and interpreter-matching
 * formatting / printing.
 *
 * It is intentionally dependency-free (libc only) so it can be cross
 * compiled together with the generated object file via `cc` or `zig cc`.
 */
#ifndef X_RUNTIME_XRT_H
#define X_RUNTIME_XRT_H

#include <stddef.h>

typedef enum {
    X_INT = 0,
    X_DOUBLE = 1,
    X_BOOL = 2,
    X_CHAR = 3,
    X_STR = 4,
    X_PTR = 5,
    X_LIST = 6,
    X_MAP = 7,
    X_UNIT = 8,
} XTag;

typedef struct XValue XValue;

/* Boxing constructors. Booleans/chars are passed as i64 from codegen. */
XValue *x_from_int(long long v);
XValue *x_from_double(double v);
XValue *x_from_bool(long long v);
XValue *x_from_char(long long v);
XValue *x_from_str(const char *s);
XValue *x_from_ptr(void *p);

/* Lists (heterogeneous, boxed values). */
XValue *x_list_new(void);
void x_list_push(XValue *list, XValue *item);
XValue *x_list_get(XValue *list, long long index);
long long x_list_len(XValue *list);

/* Maps (insertion-ordered, boxed keys/values). */
XValue *x_map_new(void);
void x_map_put(XValue *map, XValue *key, XValue *value);

/* Unboxing accessors: extract the underlying representation from a boxed
 * XValue (used when an enum payload is projected back to its real type). */
long long x_as_int(XValue *v);
double x_as_double(XValue *v);
long long x_as_bool(XValue *v);
char *x_as_str(XValue *v);
void *x_as_ptr(XValue *v);

/* Formatting / conversion. Returned strings are heap-allocated (leaked). */
char *x_fmt_value(XValue *v);
char *x_to_str(XValue *v);
char *x_str_concat(const char *a, const char *b);

/* Printing. */
void x_print(XValue *v);        /* value + newline */
void x_print_inline(XValue *v); /* value, no newline */
void x_print_newline(void);

#endif /* X_RUNTIME_XRT_H */
